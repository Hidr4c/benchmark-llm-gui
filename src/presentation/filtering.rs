use std::collections::HashSet;
use crate::domain::hardware::HardwarePlatform;
use crate::domain::license::LicenseCategory;
use crate::domain::modality::ModelModality;
use crate::domain::model::{ModelBenchmark, SortField, SortOrder, SourceSelection};

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
            Self::All => "🌐 All Types",
            Self::Specific(m) => m.label(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableColumnMode {
    #[default]
    GlobalEvaluation,
    StrictVerified,
    RawBenchmarkSources,
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
            group_by_root: true, // Group by root model family by default
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
        }
    }
}

impl FilterState {
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

fn cmp_f32(a: f32, b: f32, order: SortOrder) -> std::cmp::Ordering {
    match order {
        SortOrder::Descending => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
        SortOrder::Ascending => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
    }
}

fn cmp_ord<T: Ord>(a: T, b: T, order: SortOrder) -> std::cmp::Ordering {
    match order {
        SortOrder::Descending => b.cmp(&a),
        SortOrder::Ascending => a.cmp(&b),
    }
}

pub struct FilterService;

impl FilterService {
    pub fn compute_filtered_indices(models: &[ModelBenchmark], filters: &FilterState) -> Vec<usize> {
        let mut indices: Vec<usize> = models.iter().enumerate().filter_map(|(idx, m)| {
            if !filters.search_text.is_empty() {
                let q = filters.search_text.to_lowercase();
                let matches_name = m.name.to_lowercase().contains(&q);
                let matches_id = m.id.to_lowercase().contains(&q);
                let matches_fam = m.family.to_lowercase().contains(&q);
                if !matches_name && !matches_id && !matches_fam {
                    return None;
                }
            }

            match filters.scope {
                ModelScope::LocalOnly if m.is_cloud => return None,
                ModelScope::CloudOnly if !m.is_cloud => return None,
                _ => {}
            }

            if let ModalityFilter::Specific(target_mod) = filters.modality {
                if m.modality != target_mod {
                    return None;
                }
            }

            if filters.license != LicenseCategory::All {
                if m.license().category() != filters.license {
                    return None;
                }
            }

            if !m.is_cloud && filters.hardware != HardwarePlatform::All {
                if !filters.hardware.fits_model(m.size_gb, m.vram_rec_gb) {
                    return None;
                }
            }

            if !m.is_cloud && (m.size_gb < filters.min_size_gb || m.size_gb > filters.max_size_gb) {
                return None;
            }

            if filters.min_speed_tps > 0.0 && m.tokens_per_sec.unwrap_or(0.0) < filters.min_speed_tps {
                return None;
            }

            if filters.min_intelligence > 0.0 && m.score_for_sources(&filters.sources).unwrap_or(0.0) < filters.min_intelligence {
                return None;
            }

            if filters.selected_family != "All" && filters.selected_family != "Tous" && m.family != filters.selected_family {
                return None;
            }

            if filters.column_mode == TableColumnMode::StrictVerified {
                if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                    if breakdown.confidence == crate::domain::scoring::ScoreConfidence::Low {
                        return None;
                    }
                } else {
                    return None;
                }
            }

            Some(idx)
        }).collect();

        let order = filters.sort_order;

        match filters.sort_field {
            SortField::Intelligence => indices.sort_by(|&a, &b| cmp_f32(models[a].score_for_sources(&filters.sources).unwrap_or(0.0), models[b].score_for_sources(&filters.sources).unwrap_or(0.0), order)),
            SortField::Efficiency => indices.sort_by(|&a, &b| cmp_f32(models[a].efficiency_ratio_with_sources(&filters.sources), models[b].efficiency_ratio_with_sources(&filters.sources), order)),
            SortField::Speed => indices.sort_by(|&a, &b| cmp_f32(models[a].tokens_per_sec.unwrap_or(0.0), models[b].tokens_per_sec.unwrap_or(0.0), order)),
            SortField::Size => indices.sort_by(|&a, &b| cmp_f32(models[a].size_gb, models[b].size_gb, order)),
            SortField::Memory => indices.sort_by(|&a, &b| cmp_f32(models[a].vram_rec_gb, models[b].vram_rec_gb, order)),
            SortField::Downloads => indices.sort_by(|&a, &b| cmp_ord(models[a].downloads, models[b].downloads, order)),
            SortField::Name => indices.sort_by(|&a, &b| cmp_ord(models[a].name.to_lowercase(), models[b].name.to_lowercase(), order)),
            SortField::Price => indices.sort_by(|&a, &b| cmp_f32(models[a].pricing_input_usd.unwrap_or(0.0), models[b].pricing_input_usd.unwrap_or(0.0), order)),
            SortField::Year => indices.sort_by(|&a, &b| cmp_ord(models[a].release_year.unwrap_or(0), models[b].release_year.unwrap_or(0), order)),
            SortField::RealWorld => indices.sort_by(|&a, &b| cmp_f32(
                models[a].economic.as_ref().and_then(|e| e.composite_score()).or(models[a].scores.real_world).unwrap_or(0.0),
                models[b].economic.as_ref().and_then(|e| e.composite_score()).or(models[b].scores.real_world).unwrap_or(0.0),
                order
            )),
            SortField::Coding => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.coding.unwrap_or(0.0), models[b].scores.coding.unwrap_or(0.0), order)),
            SortField::Agentic => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.agentic.unwrap_or(0.0), models[b].scores.agentic.unwrap_or(0.0), order)),
        }

        indices
    }

    pub fn group_into_roots(models: &[ModelBenchmark], indices: &[usize]) -> Vec<RootModelGroup> {
        let mut groups: Vec<RootModelGroup> = Vec::new();
        let mut name_to_group: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for &idx in indices {
            let root_name = models[idx].root_model_name();
            if let Some(&group_idx) = name_to_group.get(&root_name) {
                groups[group_idx].variant_indices.push(idx);
            } else {
                let new_idx = groups.len();
                name_to_group.insert(root_name.clone(), new_idx);
                groups.push(RootModelGroup {
                    root_name,
                    family: models[idx].family.clone(),
                    champion_idx: idx,
                    variant_indices: vec![idx],
                });
            }
        }

        groups
    }
}
