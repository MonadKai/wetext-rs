//! Frozen Chinese 1.2.0 grammar execution, independent of legacy auto detection.

use crate::error::Result;
use crate::text_normalizer::FstTextNormalizer;
use crate::token_parser::TokenParser;
use std::path::Path;
use std::sync::Arc;

/// Shared immutable base graphs; each clone is safe for a separate mutable worker.
#[derive(Clone)]
pub struct CompiledChineseItn {
    tagger: Arc<FstTextNormalizer>,
    verbalizer: Arc<FstTextNormalizer>,
}

impl CompiledChineseItn {
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
}
