//! Oracle Unicode 15 case/category policy with original UTF-8 byte alignment.
use crate::error::{Result, WeTextError};
use serde::Deserialize;
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

#[derive(Deserialize)]
pub(super) struct UnicodePolicy {
    version: String,
    upper: BTreeMap<char, String>,
    ranges: Vec<[u32; 3]>,
}

impl UnicodePolicy {
    pub fn load() -> Result<Self> {
        let policy: Self = serde_json::from_str(include_str!(
            "../../profiles/number_plate_v1_5/unicode-15.json"
        ))
        .map_err(|e| WeTextError::InvalidProfile(e.to_string()))?;
        if policy.version != "15.0.0" {
            return Err(WeTextError::InvalidProfile("Unicode policy version".into()));
        }
        Ok(policy)
    }
    fn flags(&self, c: char) -> u32 {
        let n = c as u32;
        let i = self.ranges.partition_point(|range| range[1] < n);
        self.ranges
            .get(i)
            .filter(|range| range[0] <= n)
            .map_or(0, |range| range[2])
    }
    pub fn upper(&self, text: &str) -> String {
        let mut output = String::new();
        for c in text.chars() {
            if let Some(value) = self.upper.get(&c) {
                output.push_str(value);
            } else {
                output.push(c);
            }
        }
        output
    }
    pub fn view(&self, text: &str) -> String {
        let mut output = String::new();
        let mut start = 0;
        // Frozen C characters are normalization barriers, including codepoints
        // assigned only in a newer host Unicode version. Do not normalize them away.
        for (i, c) in text.char_indices() {
            if self.flags(c) & 4 != 0 {
                self.append(&text[start..i], &mut output);
                start = i + c.len_utf8();
            }
        }
        self.append(&text[start..], &mut output);
        output
    }
    fn append(&self, text: &str, output: &mut String) {
        for c in text.nfkc().flat_map(|c| {
            self.upper
                .get(&c)
                .map_or_else(|| c.to_string(), Clone::clone)
                .chars()
                .collect::<Vec<_>>()
        }) {
            let flags = self.flags(c);
            if flags & 2 == 0 {
                output.push(if flags & 1 != 0 { '|' } else { c });
            }
        }
    }
    pub fn offsets(&self, text: &str, expected: &str) -> Option<Vec<(usize, usize)>> {
        let mut rendered = String::new();
        let mut offsets = Vec::new();
        for (start, c) in text.char_indices() {
            let part = self.view(&c.to_string());
            rendered.push_str(&part);
            offsets.extend(std::iter::repeat_n(
                (start, start + c.len_utf8()),
                part.len(),
            ));
        }
        (rendered == expected).then_some(offsets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    #[test]
    fn complete_scalar_view_matches_independent_python15_digest() {
        let policy = UnicodePolicy::load().unwrap();
        let raw: String = (0..=0x10ffff).filter_map(char::from_u32).collect();
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../profiles/number_plate_v1_5/unicode-oracle.json"
        ))
        .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(raw.as_bytes())),
            expected["input_sha256"].as_str().unwrap()
        );
        let view = policy.view(&raw);
        assert_eq!(
            format!("{:x}", Sha256::digest(view.as_bytes())),
            expected["view_sha256"].as_str().unwrap()
        );
        assert_eq!(
            view.len(),
            expected["view_bytes"].as_u64().unwrap() as usize
        );
    }
    #[test]
    fn expansion_uses_original_bytes_and_cross_character_composition_fails_closed() {
        let policy = UnicodePolicy::load().unwrap();
        let text = "ﬃ苏Ｂ１２３４５";
        assert_eq!(policy.view(text), "FFI苏B12345");
        assert_eq!(policy.offsets(text, &policy.view(text)).unwrap()[0], (0, 3));
        assert!(policy
            .offsets("e\u{301}苏B12345", &policy.view("e\u{301}苏B12345"))
            .is_none());
        assert_eq!(policy.view("e\u{1c89}\u{301}"), "E\u{301}");
    }
}
