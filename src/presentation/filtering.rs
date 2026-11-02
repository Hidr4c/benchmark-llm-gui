use std::collections::HashSet;
use crate::domain::hardware::HardwarePlatform;
use crate::domain::license::LicenseCategory;
use crate::domain::modality::ModelModality;
use crate::domain::model::{SortField, SortOrder, SourceSelection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelScope {
    LocalOnly,  // Local models (PC & Embedded - 100% Free & Private)
    CloudOnly,  // Cloud models (Paid APIs: GPT, Claude, Gemini...)
    All,        // All models (Compare Local vs Cloud)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPriority {
    Intelligence, // Highest intelligence score
    Efficiency,   // Best intelligence-to-size efficiency
    Speed,        // Maximum throughput speed
    Compactness,  // Minimal memory footprint
    RealWorld,    // Best Real-World / Economic Value score
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeCategory {
    All,
    MicroIot,       // <= 500 MB (ESP32-P4 / Micro ARM)
    EmbeddedPi,     // 500 MB - 2 GB (Raspberry Pi 4 / 3)
    AdvancedSbc,    // 2 GB - 5 GB (Raspberry Pi 5 / Jetson Orin Nano)
    DesktopGpu,     // 5 GB - 18 GB (RTX 4060 - 4080)
    HighEndGpu,     // 18 GB - 32 GB (RTX 4090 / 5090)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalityFilter {
    #[default]
    All,
    Specific(ModelModality),
}

impl ModalityFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => " All Types",
            Self::Specific(m) => m.label(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableColumnMode {
    #[default]
    GlobalEvaluation,
    DetailedEvaluation,
    StrictVerified,
    RawBenchmarkSources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfidenceFilter {
    #[default]
    All,
    MediumPlus,
    HighOnly,
}

impl ConfidenceFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "Tous les niveaux",
            Self::MediumPlus => "Moyenne & +",
            Self::HighOnly => "Haute (3+ sources)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootModelGroup {
    pub root_name: String,
    pub family: String,
    pub champion_idx: usize,
    pub variant_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub scope: ModelScope,
    pub hardware: HardwarePlatform,
    pub priority: UserPriority,
    pub sources: SourceSelection,
    pub modality: ModalityFilter,
    pub column_mode: TableColumnMode,
    pub show_advanced_columns: bool,
    pub show_raw_sources: bool,
    pub confidence_level: ConfidenceFilter,
    pub group_by_root: bool,
    pub expanded_roots: HashSet<String>,
    pub show_advanced: bool,
    pub size_category: SizeCategory,
    pub license: LicenseCategory,
    pub search_text: String,
    pub min_size_gb: f32,
    pub max_size_gb: f32,
    pub min_speed_tps: f32,
    pub min_intelligence: f32,
    pub selected_family: String,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub compared_model_ids: Vec<String>,
    pub table_prefs: crate::domain::table_preferences::TablePreferences,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            scope: ModelScope::All, // All models (Frontier Cloud & Local Flagships)
            hardware: HardwarePlatform::All,
            priority: UserPriority::Intelligence,
            sources: SourceSelection::default(),
            modality: ModalityFilter::All,
            column_mode: TableColumnMode::GlobalEvaluation,
            show_advanced_columns: false,
            show_raw_sources: false,
            confidence_level: ConfidenceFilter::All,
            group_by_root: true,
            expanded_roots: HashSet::new(),
            show_advanced: false,
            size_category: SizeCategory::All,
            license: LicenseCategory::All,
            search_text: String::new(),
            min_size_gb: 0.0,
            max_size_gb: 60.0,
            min_speed_tps: 0.0,
            min_intelligence: 0.0,
            selected_family: "All".to_string(),
            sort_field: SortField::Intelligence,
            sort_order: SortOrder::Descending,
            compared_model_ids: Vec::new(),
            table_prefs: crate::domain::table_preferences::TablePreferences::default(),
        }
    }
}

impl FilterState {
    pub fn toggle_compare(&mut self, id: &str) {
        if let Some(pos) = self.compared_model_ids.iter().position(|m| m == id) {
            self.compared_model_ids.remove(pos);
        } else {
            if self.compared_model_ids.len() >= 2 {
                self.compared_model_ids.remove(0);
            }
            self.compared_model_ids.push(id.to_string());
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_priority(&mut self, priority: UserPriority) {
        self.priority = priority;
        match priority {
            UserPriority::Intelligence => { self.sort_field = SortField::Intelligence; self.sort_order = SortOrder::Descending; }
            UserPriority::Efficiency => { self.sort_field = SortField::Efficiency; self.sort_order = SortOrder::Descending; }
            UserPriority::Speed => { self.sort_field = SortField::Speed; self.sort_order = SortOrder::Descending; }
            UserPriority::Compactness => { self.sort_field = SortField::Size; self.sort_order = SortOrder::Ascending; }
            UserPriority::RealWorld => { self.sort_field = SortField::RealWorld; self.sort_order = SortOrder::Descending; }
        }
    }

    pub fn set_size_category(&mut self, cat: SizeCategory) {
        self.size_category = cat;
        match cat {
            SizeCategory::All => { self.min_size_gb = 0.0; self.max_size_gb = 60.0; }
            SizeCategory::MicroIot => { self.min_size_gb = 0.0; self.max_size_gb = 0.55; }
            SizeCategory::EmbeddedPi => { self.min_size_gb = 0.45; self.max_size_gb = 2.0; }
            SizeCategory::AdvancedSbc => { self.min_size_gb = 2.0; self.max_size_gb = 5.0; }
            SizeCategory::DesktopGpu => { self.min_size_gb = 5.0; self.max_size_gb = 18.0; }
            SizeCategory::HighEndGpu => { self.min_size_gb = 18.0; self.max_size_gb = 35.0; }
        }
    }
}

pub use crate::presentation::filtering_service::FilterService;
