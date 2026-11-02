pub mod en;
pub mod fr;
pub mod language;
pub mod locale;
pub mod sub_translations;
pub mod sub_translations_features;
pub mod sub_translations_tutorial;

pub use language::Language;
pub use locale::Translations;
pub use sub_translations::{
    AdvFilterTranslations, ComparatorTranslations, ConfidenceTranslations, DockTranslations,
    UpdateTranslations,
};
pub use sub_translations_features::{
    ColumnModeTranslations, RoiTranslations, RunnerTranslations,
};
pub use sub_translations_tutorial::TutorialGuideTranslations;
