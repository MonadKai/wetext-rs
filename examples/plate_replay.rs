//! JSONL qualification runner: algorithms see only original text, never IDs/gold.
use serde::Deserialize;
use std::io::{self, BufRead, Write};
use std::path::Path;
use wetext_rs::{plate::PlateProfile, CompiledChineseItn};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    id: String,
    text: String,
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut profile =
        PlateProfile::structured(include_str!("../profiles/number_plate_v1_5/lexicon.json"))?;
    if !args.is_empty() {
        anyhow::ensure!(
            args.len() == 3 && matches!(args[0].as_str(), "--full" | "--full-request"),
            "usage: plate_replay [--full|--full-request tagger.fst verbalizer.fst]"
        );
        let base = CompiledChineseItn::from_files(Path::new(&args[1]), Path::new(&args[2]))?;
        profile = if args[0] == "--full-request" {
            profile.with_request_base(base)
        } else {
            profile.with_base(base)
        };
    }
    let mut ids = std::collections::HashSet::new();
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for line in io::stdin().lock().lines() {
        let input: Input = serde_json::from_str(&line?)?;
        anyhow::ensure!(ids.insert(input.id.clone()), "duplicate input ID");
        anyhow::ensure!(input.text.len() <= 65536, "input byte limit exceeded");
        let (result, spans) = profile.normalize_with_plan(&input.text)?;
        serde_json::to_writer(
            &mut output,
            &serde_json::json!({"id":input.id,"result":result,"spans":spans}),
        )?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    Ok(())
}
