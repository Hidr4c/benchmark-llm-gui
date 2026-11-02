use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};

pub use crate::domain::table_column::TableColumnId;
pub use crate::domain::table_filter::{ColumnFilter, FilterOperator};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TablePreferences {
    pub column_order: Vec<TableColumnId>,
    pub hidden_columns: HashSet<TableColumnId>,
    pub column_filters: HashMap<TableColumnId, ColumnFilter>,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
}

impl Default for TablePreferences {
    fn default() -> Self {
        Self {
            column_order: TableColumnId::default_order(),
            hidden_columns: HashSet::new(),
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

    pub fn is_visible(&self, col: TableColumnId) -> bool {
        !self.hidden_columns.contains(&col)
    }

    pub fn toggle_visibility(&mut self, col: TableColumnId) {
        if self.hidden_columns.contains(&col) {
            self.hidden_columns.remove(&col);
        } else {
            self.hidden_columns.insert(col);
        }
    }

    pub fn show_all_columns(&mut self) {
        self.hidden_columns.clear();
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
            if col == TableColumnId::Rank {
                continue; // Evaluated post-sorting in FilterService!
            }
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
