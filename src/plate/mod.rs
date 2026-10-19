//! number_plate_v1.0.0 semantics/spans/renderer owned by this fork.
//! Ported from delivered WeTextProcessing 1.2.0+plate.1 (Apache-2.0).
mod semantics;
mod spans;
mod unicode;

use crate::{error::Result, CompiledChineseItn};
use semantics::Semantics;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Semantic behavior identity, separate from executable and result schema.
pub const PLATE_SEMANTIC_VERSION: &str = "plate_semantic_v1_5";

/// Accepted/literal plan with original UTF-8 byte coordinates, never guessed offsets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlateSpan {
    pub start: usize,
    pub end: usize,
    pub raw: String,
    pub output: String,
    pub accepted: bool,
    pub preserved_context: String,
}

/// Delivered four-field result from one accepted span plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlateResult {
    pub text: String,
    pub plate: Option<String>,
    pub plates: Vec<String>,
    pub version: String,
}

/// Immutable profile semantics plus worker-local base executor handle.
#[derive(Clone)]
pub struct PlateProfile {
    semantics: Arc<Semantics>,
    base: Option<CompiledChineseItn>,
}

impl PlateProfile {
    /// Exact lexicon bytes consumed by this instance.
    pub fn lexicon_sha256(&self) -> &str {
        &self.semantics.lexicon_sha256
    }
    /// Effective profile parameters owned alongside rules and rendering order.
    pub fn parameters(&self) -> serde_json::Value {
        use sha2::Digest;
        let mut parameters = serde_json::json!({"rendering":if self.has_full_text() {"full_text"} else {"structured_only"},
            "unicode":"15.0.0","unicode_policy_sha256":format!("{:x}",sha2::Sha256::digest(include_bytes!("../../profiles/number_plate_v1_5/unicode-15.json"))),
            "semantic_version":PLATE_SEMANTIC_VERSION,"max_explicit_kind_nesting":64});
        if self.has_full_text() {
            parameters["base"] = CompiledChineseItn::delivery_parameters();
        }
        parameters
    }
    /// Structured-only profile: unrelated text is preserved, not generic ITN.
    pub fn structured(lexicon_json: &str) -> Result<Self> {
        Ok(Self {
            semantics: Arc::new(Semantics::new(lexicon_json)?),
            base: None,
        })
    }
    /// Bind exact base graphs for complete rendering order. Never inferred at runtime.
    pub fn with_base(mut self, base: CompiledChineseItn) -> Self {
        self.base = Some(base);
        self
    }
    /// Whether this instance includes generic full-sentence ITN.
    pub fn has_full_text(&self) -> bool {
        self.base.is_some()
    }
    /// Resolve the final accepted/literal plan independently from parser status.
    pub fn plan(&self, raw: &str) -> Result<Vec<PlateSpan>> {
        self.semantics.spans(raw)
    }
    /// Process exactly one plan; fields are never extracted from rendered text.
    pub fn normalize(&mut self, raw: &str) -> Result<PlateResult> {
        let plan = self.plan(raw)?;
        self.render(raw, &plan)
    }
    /// Render an engine-produced plan, also usable by a diagnostic caller.
    pub fn normalize_with_plan(&mut self, raw: &str) -> Result<(PlateResult, Vec<PlateSpan>)> {
        let plan = self.plan(raw)?;
        let result = self.render(raw, &plan)?;
        Ok((result, plan))
    }
    fn render(&mut self, raw: &str, plan: &[PlateSpan]) -> Result<PlateResult> {
        let mut text = String::new();
        let mut plates = Vec::new();
        let mut pos = 0;
        for span in plan {
            text.push_str(&self.base_text(&raw[pos..span.start])?);
            text.push_str(&span.output);
            text.push_str(&self.base_text(&span.preserved_context)?);
            if span.accepted && !plates.contains(&span.output) {
                plates.push(span.output.clone());
            }
            pos = span.end;
        }
        text.push_str(&self.base_text(&raw[pos..])?);
        Ok(PlateResult {
            text,
            plate: if plates.len() == 1 {
                plates.first().cloned()
            } else {
                None
            },
            plates,
            version: PLATE_SEMANTIC_VERSION.into(),
        })
    }
    fn base_text(&mut self, text: &str) -> Result<String> {
        match &mut self.base {
            Some(base) => base.normalize(text),
            None => Ok(text.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deeply_nested_symbol_kinds_are_execution_errors_not_stack_overflows() {
        let mut engine = PlateProfile::structured(include_str!(
            "../../profiles/number_plate_v1_5/lexicon.json"
        ))
        .unwrap();
        assert!(matches!(
            engine.normalize(&format!("苏B{}J一二三四", "字母".repeat(65))),
            Err(crate::error::WeTextError::InvalidInput(_))
        ));
        assert_eq!(
            engine
                .normalize(&format!("苏B{}J一二三四", "字母".repeat(64)))
                .unwrap()
                .plate,
            Some("苏BJ1234".into())
        );
        assert_eq!(
            engine.normalize("苏B丁钩一二三四").unwrap().plate,
            Some("苏BJ1234".into())
        );
    }
    #[test]
    fn mutable_workers_and_distinct_lexicons_do_not_cross_contaminate() {
        let lexicon = include_str!("../../profiles/number_plate_v1_5/lexicon.json");
        let original = PlateProfile::structured(lexicon).unwrap();
        let mut other: serde_json::Value = serde_json::from_str(lexicon).unwrap();
        other["letter_aliases"]["K"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("丁钩"));
        let alternate = PlateProfile::structured(&other.to_string()).unwrap();
        let threads: Vec<_> = (0..4)
            .map(|i| {
                let mut engine = if i % 2 == 0 {
                    original.clone()
                } else {
                    alternate.clone()
                };
                std::thread::spawn(move || {
                    for _ in 0..50 {
                        let result = engine.normalize("苏B丁钩一二三四").unwrap();
                        assert_eq!(
                            result.plate,
                            if i % 2 == 0 {
                                Some("苏BJ1234".into())
                            } else {
                                None
                            }
                        );
                        assert_eq!(
                            engine.normalize("苏B三个六七八").unwrap().plate,
                            Some("苏B66678".into())
                        );
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
    #[test]
    fn structured_fields_and_full_span_contract_match_frozen_oracle_cases() {
        let mut engine = PlateProfile::structured(include_str!(
            "../../profiles/number_plate_v1_5/lexicon.json"
        ))
        .unwrap();
        let inputs = include_str!("../../profiles/number_plate_v1_5/oracle-inputs.jsonl").lines();
        let expected =
            include_str!("../../profiles/number_plate_v1_5/oracle-outputs.jsonl").lines();
        let mut count = 0;
        for (input, expected) in inputs.zip(expected) {
            let input: serde_json::Value = serde_json::from_str(input).unwrap();
            let expected: serde_json::Value = serde_json::from_str(expected).unwrap();
            let (result, spans) = engine
                .normalize_with_plan(input["text"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                serde_json::to_value(&result.plate).unwrap(),
                expected["result"]["plate"],
                "{}",
                input["id"]
            );
            assert_eq!(
                serde_json::to_value(&result.plates).unwrap(),
                expected["result"]["plates"],
                "{}",
                input["id"]
            );
            assert_eq!(
                serde_json::to_value(spans).unwrap(),
                expected["spans"],
                "{}",
                input["id"]
            );
            count += 1;
        }
        assert_eq!(count, 83);
    }
}
