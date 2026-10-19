use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::ApiError;

/// Plugin metadata, corresponding to Java's `ForumPlugin` descriptive fields.
pub struct PluginMeta {
    pub key: &'static str,
    pub name: &'static str,
    /// REVIEW | DOCUMENT_REVIEW | RECOMMENDATION | CONTENT_BLOCK
    pub ptype: &'static str,
    pub enabled_default: bool,
    pub description: &'static str,
}

/// All built-in plugins, sorted ascending by key (matching Java's `PluginRegistry` construction order).
pub const PLUGINS: [PluginMeta; 5] = [
    PluginMeta {
        key: "ai-review",
        name: "AI Review",
        ptype: "REVIEW",
        enabled_default: false,
        description: "Uses an OpenAI-compatible API for intelligent review; falls back to the rule engine when no key is configured",
    },
    PluginMeta {
        key: "ai-vision-review",
        name: "AI Document Vision Review",
        ptype: "DOCUMENT_REVIEW",
        enabled_default: false,
        description: "Uses a multimodal model to review document images and extract identity details; skipped when no key is configured",
    },
    PluginMeta {
        key: "heuristic-document",
        name: "Heuristic Document Validation",
        ptype: "DOCUMENT_REVIEW",
        enabled_default: true,
        description: "Checks document image count, decodability, and resolution; rejects clear failures and routes the rest to manual review",
    },
    PluginMeta {
        key: "hot-score-recommendation",
        name: "Hot Score Recommendation",
        ptype: "RECOMMENDATION",
        enabled_default: true,
        description: "Computes a sub-forum recommendation score from member count, post count, and activity over the last 7 days",
    },
    PluginMeta {
        key: "rule-based-review",
        name: "Rule-based Review Engine",
        ptype: "REVIEW",
        enabled_default: true,
        description: "Validates required fields and formats for each verification type (18-digit ID, age 0-120, non-empty nationality, etc.)",
    },
];

/// Runtime toggles (in-memory, matching Java: defaults are restored after a restart).
pub struct PluginRegistry {
    overrides: RwLock<HashMap<&'static str, bool>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            overrides: RwLock::new(HashMap::new()),
        }
    }

    pub fn all(&self) -> &'static [PluginMeta] {
        &PLUGINS
    }

    pub fn by_key(&self, key: &str) -> Option<&'static PluginMeta> {
        PLUGINS.iter().find(|p| p.key == key)
    }

    pub fn is_enabled(&self, meta: &PluginMeta) -> bool {
        self.overrides
            .read()
            .unwrap()
            .get(meta.key)
            .copied()
            .unwrap_or(meta.enabled_default)
    }

    pub fn is_enabled_key(&self, key: &str) -> bool {
        match self.by_key(key) {
            Some(meta) => self.is_enabled(meta),
            None => false,
        }
    }

    pub fn set_enabled(&self, key: &str, enabled: bool) -> Result<(), ApiError> {
        let meta = self
            .by_key(key)
            .ok_or_else(|| ApiError::not_found(format!("Plugin not found: {key}")))?;
        self.overrides.write().unwrap().insert(meta.key, enabled);
        Ok(())
    }

    pub fn enabled_document_reviews(&self) -> Vec<&'static PluginMeta> {
        PLUGINS
            .iter()
            .filter(|p| p.ptype == "DOCUMENT_REVIEW" && self.is_enabled(p))
            .collect()
    }
}
