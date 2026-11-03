use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseCategory {
    All,
    PermissiveCommercial, // Apache 2.0, MIT, OpenRAIL
    CommunityCommercial,  // Llama Community, Gemma Terms, Qwen
    CloudProprietary,     // Closed proprietary API
}

impl LicenseCategory {
    pub fn all() -> [Self; 4] {
        [
            Self::All,
            Self::PermissiveCommercial,
            Self::CommunityCommercial,
            Self::CloudProprietary,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelLicense {
    Apache2,
    Mit,
    LlamaCommunity,
    GemmaTerms,
    OpenRail,
    ProprietaryCloud,
    OtherOpen,
}

impl ModelLicense {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Apache2 => "Apache 2.0 (Commercial Libre)",
            Self::Mit => "MIT (Libre & Sans Restriction)",
            Self::LlamaCommunity => "Llama Community (Gratuit ≤ 700M MAU)",
            Self::GemmaTerms => "Gemma Terms (Open Commercial)",
            Self::OpenRail => "OpenRAIL-M (Recherche & Prod)",
            Self::ProprietaryCloud => "Propriétaire (API Payante)",
            Self::OtherOpen => "Open Weights / Custom",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Apache2 => "Apache-2.0",
            Self::Mit => "MIT",
            Self::LlamaCommunity => "Llama-3",
            Self::GemmaTerms => "Gemma",
            Self::OpenRail => "OpenRAIL",
            Self::ProprietaryCloud => "Proprietary",
            Self::OtherOpen => "Open-Weight",
        }
    }

    pub fn category(&self) -> LicenseCategory {
        match self {
            Self::Apache2 | Self::Mit | Self::OpenRail => LicenseCategory::PermissiveCommercial,
            Self::LlamaCommunity | Self::GemmaTerms | Self::OtherOpen => LicenseCategory::CommunityCommercial,
            Self::ProprietaryCloud => LicenseCategory::CloudProprietary,
        }
    }

    pub fn is_permissive_commercial(&self) -> bool {
        matches!(self, Self::Apache2 | Self::Mit | Self::OpenRail)
    }

    pub fn detect(id: &str, provider: Option<&str>, is_cloud: bool) -> Self {
        if is_cloud || provider.is_some_and(|p| p == "OpenAI" || p == "Anthropic" || p == "Google") {
            return Self::ProprietaryCloud;
        }

        let lower = id.to_lowercase();
        if lower.contains("deepseek") || lower.contains("phi") {
            Self::Mit
        } else if lower.contains("qwen") || lower.contains("smollm") || lower.contains("mistral") {
            Self::Apache2
        } else if lower.contains("llama") {
            Self::LlamaCommunity
        } else if lower.contains("gemma") {
            Self::GemmaTerms
        } else if lower.contains("starcoder") || lower.contains("bigcode") {
            Self::OpenRail
        } else {
            Self::OtherOpen
        }
    }
}
