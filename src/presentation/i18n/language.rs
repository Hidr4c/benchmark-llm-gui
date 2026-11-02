use serde::{Deserialize, Serialize};
use super::en::EN_TRANSLATIONS;
use super::fr::FR_TRANSLATIONS;
use super::locale::Translations;
use super::sub_translations::{
    AdvFilterTranslations, ConfidenceTranslations, DockTranslations,
    EN_ADV_FILTER, EN_CONFIDENCE, EN_DOCK,
    FR_ADV_FILTER, FR_CONFIDENCE, FR_DOCK,
};
use super::sub_translations_features::{
    ColumnModeTranslations, RoiTranslations, RunnerTranslations,
    EN_COL_MODES, EN_ROI, EN_RUNNER,
    FR_COL_MODES, FR_ROI, FR_RUNNER,
};
use super::sub_translations_tools::{
    EN_TOOLS_GUIDE, FR_TOOLS_GUIDE, LocalToolsTranslations,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    French,
    English,
}

impl Language {
    pub fn all() -> [Self; 2] {
        [Self::French, Self::English]
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::French => "fr",
            Self::English => "en",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::French => "FR",
            Self::English => "EN",
        }
    }

    pub fn t(&self) -> &'static Translations {
        match self {
            Self::French => &FR_TRANSLATIONS,
            Self::English => &EN_TRANSLATIONS,
        }
    }

    pub fn adv(&self) -> &'static AdvFilterTranslations {
        match self {
            Self::French => &FR_ADV_FILTER,
            Self::English => &EN_ADV_FILTER,
        }
    }

    pub fn confidence(&self) -> &'static ConfidenceTranslations {
        match self {
            Self::French => &FR_CONFIDENCE,
            Self::English => &EN_CONFIDENCE,
        }
    }

    pub fn dock(&self) -> &'static DockTranslations {
        match self {
            Self::French => &FR_DOCK,
            Self::English => &EN_DOCK,
        }
    }

    pub fn col_modes(&self) -> &'static ColumnModeTranslations {
        match self {
            Self::French => &FR_COL_MODES,
            Self::English => &EN_COL_MODES,
        }
    }

    pub fn runner(&self) -> &'static RunnerTranslations {
        match self {
            Self::French => &FR_RUNNER,
            Self::English => &EN_RUNNER,
        }
    }

    pub fn roi(&self) -> &'static RoiTranslations {
        match self {
            Self::French => &FR_ROI,
            Self::English => &EN_ROI,
        }
    }

    pub fn tools(&self) -> &'static LocalToolsTranslations {
        match self {
            Self::French => &FR_TOOLS_GUIDE,
            Self::English => &EN_TOOLS_GUIDE,
        }
    }
}
