use std::collections::HashMap;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableColumnId {
    Rank,
    Model,
    Year,
    TargetHardware,
    Size,
    Efficiency,
    Score,
    RealWorld,
    Speed,
    Popularity,
    MmluPro,
    SweBench,
    Agentic,
    OpenLlm,
    CloudPrice,
}

impl TableColumnId {
    pub fn all() -> &'static [TableColumnId] {
        &[
            TableColumnId::Rank,
            TableColumnId::Model,
            TableColumnId::Year,
            TableColumnId::TargetHardware,
            TableColumnId::Size,
            TableColumnId::Efficiency,
            TableColumnId::Score,
            TableColumnId::RealWorld,
            TableColumnId::Speed,
            TableColumnId::Popularity,
            TableColumnId::MmluPro,
            TableColumnId::SweBench,
            TableColumnId::Agentic,
            TableColumnId::OpenLlm,
            TableColumnId::CloudPrice,
        ]
    }

    pub fn default_order() -> Vec<TableColumnId> {
        Self::all().to_vec()
    }

    pub fn is_advanced(&self) -> bool {
        matches!(
            self,
            TableColumnId::Year
                | TableColumnId::Efficiency
                | TableColumnId::RealWorld
                | TableColumnId::Popularity
        )
    }

    pub fn is_raw_source(&self) -> bool {
        matches!(
            self,
            TableColumnId::MmluPro
                | TableColumnId::SweBench
                | TableColumnId::Agentic
                | TableColumnId::OpenLlm
        )
    }

    pub fn default_sort_field(&self) -> Option<(SortField, SortOrder)> {
        match self {
            TableColumnId::Rank => None,
            TableColumnId::Model => Some((SortField::Name, SortOrder::Ascending)),
            TableColumnId::Year => Some((SortField::Year, SortOrder::Descending)),
            TableColumnId::TargetHardware => Some((SortField::Memory, SortOrder::Ascending)),
            TableColumnId::Size => Some((SortField::Size, SortOrder::Ascending)),
            TableColumnId::Efficiency => Some((SortField::Efficiency, SortOrder::Descending)),
            TableColumnId::Score => Some((SortField::Intelligence, SortOrder::Descending)),
            TableColumnId::RealWorld => Some((SortField::RealWorld, SortOrder::Descending)),
            TableColumnId::Speed => Some((SortField::Speed, SortOrder::Descending)),
            TableColumnId::Popularity => Some((SortField::Downloads, SortOrder::Descending)),
            TableColumnId::MmluPro => None,
            TableColumnId::SweBench => Some((SortField::Coding, SortOrder::Descending)),
            TableColumnId::Agentic => Some((SortField::Agentic, SortOrder::Descending)),
            TableColumnId::OpenLlm => Some((SortField::Intelligence, SortOrder::Descending)),
            TableColumnId::CloudPrice => Some((SortField::Price, SortOrder::Ascending)),
        }
    }

    pub fn extract_value(&self, model: &ModelBenchmark, rank_str: &str) -> String {
        match self {
            TableColumnId::Rank => rank_str.to_string(),
            TableColumnId::Model => model.name.clone(),
            TableColumnId::Year => model.release_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into()),
            TableColumnId::TargetHardware => target_hardware_text(model).to_string(),
            TableColumnId::Size => {
                if model.is_cloud { "Cloud API".into() } else { format!("{:.1} Go", model.size_gb) }
            }
            TableColumnId::Efficiency => format!("{:.1}", model.efficiency_ratio()),
            TableColumnId::Score => model.intelligence().map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::RealWorld => model.economic.as_ref().and_then(|e| e.composite_score())
                .or(model.scores.real_world)
                .map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Speed => model.tokens_per_sec.map(|s| format!("{s:.0}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Popularity => model.downloads.to_string(),
            TableColumnId::MmluPro => model.scores.mmlu_pro.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::SweBench => model.scores.coding.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Agentic => model.scores.agentic.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::OpenLlm => model.scores.open_llm.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::CloudPrice => model.pricing_output_usd.map(|p| format!("${p:.3}"))
                .unwrap_or_else(|| if model.is_cloud { "Cloud API".into() } else { "Local (0.00€)".into() }),
        }
    }
}

fn target_hardware_text(model: &ModelBenchmark) -> &'static str {
    if model.is_cloud { "Cloud API" }
    else if model.size_gb <= 0.45 { "Micro-IoT (<1G)" }
    else if model.size_gb <= 1.5 { "Pi 4 (2-4G)" }
    else if model.size_gb <= 3.2 { "Pi 5 (8G)" }
    else if model.size_gb <= 5.5 { "Jetson (8G)" }
    else if model.vram_rec_gb <= 8.5 { "GPU 8 Go" }
    else if model.vram_rec_gb <= 12.5 { "GPU 12 Go" }
    else if model.vram_rec_gb <= 16.5 { "GPU 16 Go" }
    else if model.vram_rec_gb <= 24.5 { "GPU 24 Go" }
    else if model.vram_rec_gb <= 32.5 { "GPU 32 Go" }
    else { "Station 48G+" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FilterOperator {
    #[default]
    Contains,
    Exact,
    NotContains,
    MatchRegex,
    NotMatchRegex,
}

impl FilterOperator {
    pub fn all() -> &'static [FilterOperator] {
        &[
            FilterOperator::Contains,
            FilterOperator::Exact,
            FilterOperator::NotContains,
            FilterOperator::MatchRegex,
            FilterOperator::NotMatchRegex,
        ]
    }

    pub fn matches(&self, value: &str, pattern: &str) -> bool {
        let pattern_trimmed = pattern.trim();
        if pattern_trimmed.is_empty() {
            return true;
        }

        match self {
            FilterOperator::Contains => value.to_lowercase().contains(&pattern_trimmed.to_lowercase()),
            FilterOperator::Exact => value.eq_ignore_ascii_case(pattern_trimmed),
            FilterOperator::NotContains => !value.to_lowercase().contains(&pattern_trimmed.to_lowercase()),
            FilterOperator::MatchRegex => {
                RegexBuilder::new(pattern_trimmed).case_insensitive(true).build()
                    .map(|re| re.is_match(value)).unwrap_or(false)
            }
            FilterOperator::NotMatchRegex => {
                RegexBuilder::new(pattern_trimmed).case_insensitive(true).build()
                    .map(|re| !re.is_match(value)).unwrap_or(false)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnFilter {
    pub operator: FilterOperator,
    pub pattern: String,
    pub active: bool,
}

impl Default for ColumnFilter {
    fn default() -> Self {
        Self {
            operator: FilterOperator::Contains,
            pattern: String::new(),
            active: false,
        }
    }
}

impl ColumnFilter {
    pub fn new(operator: FilterOperator, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let active = !pattern.trim().is_empty();
        Self { operator, pattern, active }
    }

    pub fn matches(&self, value: &str) -> bool {
        if !self.active || self.pattern.trim().is_empty() {
            return true;
        }
        self.operator.matches(value, &self.pattern)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TablePreferences {
    pub column_order: Vec<TableColumnId>,
    pub column_filters: HashMap<TableColumnId, ColumnFilter>,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
}

impl Default for TablePreferences {
    fn default() -> Self {
        Self {
            column_order: TableColumnId::default_order(),
            column_filters: HashMap::new(),
            sort_field: SortField::Intelligence,
            sort_order: SortOrder::Descending,
        }
    }
}

impl TablePreferences {
    pub fn reset_order(&mut self) {
        self.column_order = TableColumnId::default_order();
    }

    pub fn reset_filters(&mut self) {
        self.column_filters.clear();
    }

    pub fn move_column(&mut self, col: TableColumnId, delta: isize) {
        if let Some(pos) = self.column_order.iter().position(|&c| c == col) {
            let new_pos = (pos as isize + delta).clamp(0, (self.column_order.len() - 1) as isize) as usize;
            if new_pos != pos {
                let item = self.column_order.remove(pos);
                self.column_order.insert(new_pos, item);
            }
        }
    }

    pub fn has_active_filter(&self, col: TableColumnId) -> bool {
        self.column_filters.get(&col).is_some_and(|f| f.active && !f.pattern.trim().is_empty())
    }

    pub fn active_filter_count(&self) -> usize {
        self.column_filters.values().filter(|f| f.active && !f.pattern.trim().is_empty()).count()
    }

    pub fn matches_model(&self, model: &ModelBenchmark, rank_str: &str) -> bool {
        for (&col, filter) in &self.column_filters {
            if filter.active && !filter.pattern.trim().is_empty() {
                let val = col.extract_value(model, rank_str);
                if !filter.matches(&val) {
                    return false;
                }
            }
        }
        true
    }
}
