use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::ApiError;

/// 插件元信息，对应 Java `ForumPlugin` 的描述性字段。
pub struct PluginMeta {
    pub key: &'static str,
    pub name: &'static str,
    /// REVIEW | DOCUMENT_REVIEW | RECOMMENDATION | CONTENT_BLOCK
    pub ptype: &'static str,
    pub enabled_default: bool,
    pub description: &'static str,
}

/// 全部内置插件，按 key 升序排列（对齐 Java `PluginRegistry` 的构造顺序）。
pub const PLUGINS: [PluginMeta; 5] = [
    PluginMeta {
        key: "ai-review",
        name: "AI 智能审核",
        ptype: "REVIEW",
        enabled_default: false,
        description: "调用 OpenAI 兼容接口进行智能审核，未配置密钥时自动回退规则引擎",
    },
    PluginMeta {
        key: "ai-vision-review",
        name: "AI 证件视觉审核",
        ptype: "DOCUMENT_REVIEW",
        enabled_default: false,
        description: "调用多模态大模型审阅证件图片并抽取实名信息，未配置密钥时自动跳过",
    },
    PluginMeta {
        key: "heuristic-document",
        name: "证件启发式校验",
        ptype: "DOCUMENT_REVIEW",
        enabled_default: true,
        description: "校验证件图片数量、可解码性与分辨率，明显不合格时驳回，否则转人工",
    },
    PluginMeta {
        key: "hot-score-recommendation",
        name: "热度推荐",
        ptype: "RECOMMENDATION",
        enabled_default: true,
        description: "按成员数 / 帖子数 / 近 7 天活跃度计算子论坛推荐分",
    },
    PluginMeta {
        key: "rule-based-review",
        name: "规则审核引擎",
        ptype: "REVIEW",
        enabled_default: true,
        description: "校验各认证类型的必填字段与格式（身份证 18 位、年龄 0-120、国籍非空等）",
    },
];

/// 运行时开关（内存态，与 Java 一致：重启后恢复默认）。
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
            .ok_or_else(|| ApiError::not_found(format!("插件不存在: {key}")))?;
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
