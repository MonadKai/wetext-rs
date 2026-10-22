//! Frozen Chinese 1.2.0 grammar execution, independent of legacy auto detection.

use crate::error::Result;
use crate::plate::PlateSpan;
use crate::text_normalizer::FstTextNormalizer;
use crate::token_parser::TokenParser;
use rustfst::algorithms::{
    closure::{closure, ClosureType},
    compose::compose,
    concat::concat,
    optimize,
    union::union,
};
use rustfst::prelude::*;
use rustfst::utils::{acceptor, transducer};
use std::path::Path;
use std::sync::Arc;

/// Shared immutable base graphs; each clone is safe for a separate mutable worker.
#[derive(Clone)]
pub struct CompiledChineseItn {
    tagger: Arc<FstTextNormalizer>,
    verbalizer: Arc<FstTextNormalizer>,
}

impl CompiledChineseItn {
    /// Asset recipe/options for this delivery contract, not inferred from arbitrary graphs.
    pub fn delivery_parameters() -> serde_json::Value {
        serde_json::json!({"remove_interjections":true,"enable_standalone_number":true,
            "enable_0_to_9":false,"enable_million":false,"enable_plate_semantics":false,
            "cache_mode":"reload_then_optimize"})
    }

    /// Load the exact buffers verified by the asset-owning Adapter, without reopening paths.
    pub fn from_bytes(tagger: &[u8], verbalizer: &[u8]) -> Result<Self> {
        Ok(Self {
            tagger: Arc::new(FstTextNormalizer::from_bytes(tagger)?),
            verbalizer: Arc::new(FstTextNormalizer::from_bytes(verbalizer)?),
        })
    }
    /// Load both exact profile assets eagerly. No language inference or fallback.
    pub fn from_files(tagger: &Path, verbalizer: &Path) -> Result<Self> {
        Ok(Self {
            tagger: Arc::new(FstTextNormalizer::from_file(tagger)?),
            verbalizer: Arc::new(FstTextNormalizer::from_file(verbalizer)?),
        })
    }

    /// Full ordered base ITN. Do not trim original fragments or repeat postprocessing.
    pub fn normalize(&mut self, text: &str) -> Result<String> {
        if text.is_empty() {
            return Ok(String::new());
        }
        let tagged = self.tagger.normalize_strict(text)?;
        let reordered = TokenParser::chinese_itn_v120().reorder_strict(&tagged)?;
        self.verbalizer.normalize_strict(&reordered)
    }

    /// Follow delivered specialize() before either shortest-path selection.
    pub(crate) fn normalize_plate_request(
        &mut self,
        raw: &str,
        plan: &[PlateSpan],
        contexts: &[String],
    ) -> Result<String> {
        if plan.is_empty() {
            return self.normalize(raw);
        }
        let (tagger, verbalizer) = request_graphs(
            raw,
            plan,
            contexts,
            self.tagger.graph(),
            self.verbalizer.graph(),
        )
        .map_err(|e| crate::error::WeTextError::FstOperationError(e.to_string()))?;
        let tagged = FstTextNormalizer::from_graph(tagger).normalize_strict(raw)?;
        let reordered = TokenParser::chinese_itn_v120().reorder_strict(&tagged)?;
        FstTextNormalizer::from_graph(verbalizer).normalize_strict(&reordered)
    }
}

type Graph = VectorFst<TropicalWeight>;

fn labels(text: &str) -> Vec<Label> {
    text.as_bytes().iter().map(|&b| b as Label).collect()
}

fn cross(input: &str, output: &str) -> Graph {
    transducer(&labels(input), &labels(output), TropicalWeight::one())
}

fn wire_escape(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn restricted_tagger(raw: &str, base: &Graph) -> anyhow::Result<Graph> {
    let input: Graph = acceptor(&labels(raw), TropicalWeight::one());
    compose::<TropicalWeight, Graph, Graph, Graph, _, _>(&input, base)
}

fn request_graphs(
    raw: &str,
    plan: &[PlateSpan],
    contexts: &[String],
    base_tagger: &Graph,
    base_verbalizer: &Graph,
) -> anyhow::Result<(Graph, Graph)> {
    anyhow::ensure!(plan.len() == contexts.len(), "context/plan length mismatch");
    let mut tagger = cross("", "");
    let mut verbalizer = base_verbalizer.clone();
    let mut pos = 0;
    for (span, context) in plan.iter().zip(contexts) {
        anyhow::ensure!(
            pos <= span.start
                && span.start <= span.end
                && raw.get(span.start..span.end) == Some(span.raw.as_str()),
            "span does not address the original snapshot"
        );
        if pos < span.start {
            concat(
                &mut tagger,
                &restricted_tagger(&raw[pos..span.start], base_tagger)?,
            )?;
            concat(&mut tagger, &cross("", " "))?;
        }
        let name = if span.accepted {
            "licenseplate_semantic"
        } else {
            "licenseplate_literal"
        };
        let value = wire_escape(&span.raw);
        let token = format!("{name} {{ value: \"{value}\" }}");
        let mut semantic_tagger = cross("", &format!("{name} {{ value: \""));
        concat(&mut semantic_tagger, &cross(&span.raw, &value))?;
        concat(&mut semantic_tagger, &cross("", "\" } "))?;
        optimize(&mut semantic_tagger)?;
        concat(&mut tagger, &semantic_tagger)?;

        let output = format!("{}{context}", span.output);
        let mut semantic_verbalizer = cross(&token, &output);
        let mut optional_space = cross("", "");
        union(&mut optional_space, &cross(" ", ""))?;
        concat(&mut semantic_verbalizer, &optional_space)?;
        optimize(&mut semantic_verbalizer)?;
        union(&mut verbalizer, &semantic_verbalizer)?;
        pos = span.end;
    }
    if pos < raw.len() {
        concat(&mut tagger, &restricted_tagger(&raw[pos..], base_tagger)?)?;
    }
    // Selecting each gap's shortest path first changes equal-weight choices.
    optimize(&mut tagger)?;
    closure(&mut verbalizer, ClosureType::ClosureStar);
    Ok((tagger, verbalizer))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_assets_are_required_at_construction() {
        assert!(CompiledChineseItn::from_files(
            Path::new("missing-tagger"),
            Path::new("missing-verbalizer")
        )
        .is_err());
    }

    #[test]
    fn delivered_wire_escape_keeps_controls_and_escapes_only_quote_and_backslash() {
        assert_eq!(wire_escape("a\\\"\n京"), "a\\\\\\\"\n京");
    }

    #[test]
    fn request_graphs_reject_invalid_original_offsets_and_context_count() {
        let graph = cross("", "");
        let span = PlateSpan {
            start: 1,
            end: 3,
            raw: "京".into(),
            output: "京".into(),
            accepted: false,
            preserved_context: String::new(),
        };
        assert!(request_graphs("京", std::slice::from_ref(&span), &[], &graph, &graph).is_err());
        assert!(request_graphs("京", &[span], &[String::new()], &graph, &graph).is_err());
    }
}
