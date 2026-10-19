//! Explicit asset qualification, including cached-graph equal-weight regressions.
use std::path::PathBuf;
use wetext_rs::{plate::PlateProfile, CompiledChineseItn};

#[test]
#[ignore = "requires WETEXT_TAGGER and WETEXT_VERBALIZER from the frozen cached export"]
fn frozen_full_text_and_cache_path_order_match_oracle() {
    let mut engine =
        PlateProfile::structured(include_str!("../profiles/number_plate_v1_5/lexicon.json"))
            .unwrap()
            .with_base(
                CompiledChineseItn::from_files(
                    &PathBuf::from(std::env::var("WETEXT_TAGGER").unwrap()),
                    &PathBuf::from(std::env::var("WETEXT_VERBALIZER").unwrap()),
                )
                .unwrap(),
            );
    let mut count = 0;
    for (inputs, outputs) in [
        (
            include_str!("../profiles/number_plate_v1_5/oracle-inputs.jsonl"),
            include_str!("../profiles/number_plate_v1_5/oracle-outputs.jsonl"),
        ),
        (
            include_str!("../profiles/number_plate_v1_5/c-cache-inputs.jsonl"),
            include_str!("../profiles/number_plate_v1_5/c-cache-outputs.jsonl"),
        ),
    ] {
        let inputs: Vec<_> = inputs
            .lines()
            .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
            .collect();
        let outputs: std::collections::HashMap<_, _> = outputs
            .lines()
            .map(|s| {
                let v: serde_json::Value = serde_json::from_str(s).unwrap();
                (v["id"].as_str().unwrap().to_owned(), v)
            })
            .collect();
        assert_eq!(inputs.len(), outputs.len());
        for input in inputs {
            let expected = &outputs[input["id"].as_str().unwrap()];
            let (result, spans) = engine
                .normalize_with_plan(input["text"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                serde_json::to_value(result).unwrap(),
                expected["result"],
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
    }
    assert_eq!(count, 94);
}
