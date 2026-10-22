//! Accepted/literal span planning, ported separately from raw parser status.
use super::{
    semantics::{Candidate, Semantics, Status, NEUTRAL},
    PlateSpan,
};
use crate::error::{Result, WeTextError};

fn span(
    raw: &str,
    offsets: &[(usize, usize)],
    start: usize,
    end: usize,
    value: Option<&str>,
) -> Result<PlateSpan> {
    let left = offsets
        .get(start)
        .ok_or_else(|| WeTextError::InvalidProfile("span start alignment".into()))?
        .0;
    let right = offsets
        .get(
            end.checked_sub(1)
                .ok_or_else(|| WeTextError::InvalidProfile("empty span".into()))?,
        )
        .ok_or_else(|| WeTextError::InvalidProfile("span end alignment".into()))?
        .1;
    let text = raw
        .get(left..right)
        .ok_or_else(|| WeTextError::InvalidProfile("invalid UTF-8 span".into()))?;
    Ok(PlateSpan {
        start: left,
        end: right,
        raw: text.into(),
        output: value.unwrap_or(text).into(),
        accepted: value.is_some(),
        preserved_context: String::new(),
    })
}

impl Semantics {
    fn consumed_end(&self, s: &str, c: &Candidate, stop: usize) -> Result<usize> {
        let mut fresh = self.scan(s, c.start).ok_or_else(|| {
            WeTextError::InvalidProfile("candidate changed during planning".into())
        })?;
        let (changed, used) = self.correct(&mut fresh, &s[c.end..stop]);
        let mut end = c.end + if changed { used } else { 0 };
        if changed {
            if let Some(negative) = self.rules.following_negative.find(&s[end..stop]) {
                end += negative.end();
            }
        }
        Ok(end)
    }

    pub fn spans(&self, raw: &str) -> Result<Vec<PlateSpan>> {
        self.check_input(raw)?;
        let result = self.parse(raw);
        let s = &result.text;
        let candidates = &result.candidates;
        let (Some(first), Some(last)) = (candidates.first(), candidates.last()) else {
            return Ok(Vec::new());
        };
        let Some(offsets) = self.unicode.offsets(raw, s) else {
            return Ok(vec![PlateSpan {
                start: 0,
                end: raw.len(),
                raw: raw.into(),
                output: raw.into(),
                accepted: false,
                preserved_context: String::new(),
            }]);
        };
        let item =
            |start, end, value: Option<String>| span(raw, &offsets, start, end, value.as_deref());
        for (i, c) in candidates.iter().enumerate() {
            let stop = candidates.get(i + 1).map_or(s.len(), |c| c.start);
            let consumed = self.consumed_end(s, c, stop)?;
            if consumed > c.end {
                let tail = &s[c.end..consumed];
                let markers: Vec<_> = self.rules.markers.find_iter(tail).collect();
                for marker in markers.iter().skip(1) {
                    let mut fresh = self.scan(s, c.start).ok_or_else(|| {
                        WeTextError::InvalidProfile("candidate unavailable".into())
                    })?;
                    let (changed, used) = self.correct(&mut fresh, &tail[..marker.start()]);
                    if changed
                        && !self
                            .rules
                            .correction_filler
                            .replace_all(&tail[used..marker.start()], "")
                            .is_empty()
                    {
                        return Ok(vec![item(
                            first.start,
                            self.consumed_end(s, last, s.len())?,
                            None,
                        )?]);
                    }
                }
                if let Some(correction) = markers.first() {
                    if !self
                        .rules
                        .correction_filler
                        .replace_all(&tail[..correction.start()], "")
                        .is_empty()
                    {
                        let mut gap_end = c.end + correction.start();
                        let gap = &s[c.end..gap_end];
                        let explicit = self.rules.explicit_plate.find(gap);
                        let competing = self.rules.competing.is_match(gap);
                        if candidates.len() == 1
                            && result.status == Status::Complete
                            && (explicit.is_some() || !competing)
                        {
                            if let Some(explicit) = explicit {
                                gap_end = c.end + explicit.start();
                            }
                            let left = offsets[c.end - 1].1;
                            let right = offsets.get(gap_end).map_or(raw.len(), |offset| offset.0);
                            let preserved = raw[left..right]
                                .trim_end_matches(|c| " ，,、；;。.!！？?\t\n".contains(c));
                            let mut accepted = item(c.start, consumed, result.plate.clone())?;
                            accepted.preserved_context = preserved.into();
                            return Ok(vec![accepted]);
                        }
                        return Ok(vec![item(
                            first.start,
                            self.consumed_end(s, last, s.len())?,
                            None,
                        )?]);
                    }
                }
            }
        }
        if candidates.len() > 1 && matches!(result.status, Status::Complete | Status::Multiple) {
            let separate = candidates
                .windows(2)
                .try_fold(false, |found, pair| -> Result<_> {
                    let bridge = &s[self.consumed_end(s, &pair[0], pair[1].start)?..pair[1].start];
                    Ok(found || !self.rules.restart_bridge.replace_all(bridge, "").is_empty())
                })?;
            if separate {
                let mut output = Vec::new();
                for (i, c) in candidates.iter().enumerate() {
                    let stop = candidates.get(i + 1).map_or(s.len(), |c| c.start);
                    let end = self.consumed_end(s, c, stop)?;
                    let original = item(c.start, end, None)?;
                    let parsed = self.parse(&original.raw);
                    let value = if !c.unresolved && c.complete {
                        parsed.plate
                    } else {
                        None
                    };
                    output.push(item(c.start, end, value)?);
                }
                return Ok(output);
            }
        }
        if result.status == Status::Complete {
            return Ok(vec![item(
                first.start,
                self.consumed_end(s, last, s.len())?,
                result.plate.clone(),
            )?]);
        }
        if result.status == Status::Multiple {
            let mut active: Vec<&Candidate> = Vec::new();
            for c in candidates {
                if let Some(previous) = active.last() {
                    let mut bridge = s[previous.end..c.start].to_string();
                    for word in NEUTRAL {
                        bridge = bridge.replace(word, "");
                    }
                    if self.rules.restart.is_match(&bridge) || previous.value == c.value {
                        active.clear();
                    }
                }
                active.push(c);
            }
            let mut output = Vec::new();
            for (i, c) in active.iter().enumerate() {
                let stop = active.get(i + 1).map_or(s.len(), |c| c.start);
                output.push(item(
                    if i == 0 { first.start } else { c.start },
                    self.consumed_end(s, c, stop)?,
                    Some(c.value.clone()),
                )?);
            }
            return Ok(output);
        }
        Ok(vec![item(
            first.start,
            self.consumed_end(s, last, s.len())?,
            None,
        )?])
    }
}
