use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FilterOperator {
    #[default]
    Contains,
    Exact,
    LessEqual,
    GreaterEqual,
    NotContains,
    MatchRegex,
    NotMatchRegex,
}

impl FilterOperator {
    pub fn all() -> &'static [FilterOperator] {
        &[
            FilterOperator::Contains,
            FilterOperator::Exact,
            FilterOperator::LessEqual,
            FilterOperator::GreaterEqual,
            FilterOperator::NotContains,
            FilterOperator::MatchRegex,
            FilterOperator::NotMatchRegex,
        ]
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            FilterOperator::Contains => "~",
            FilterOperator::Exact => "=",
            FilterOperator::LessEqual => "<=",
            FilterOperator::GreaterEqual => ">=",
            FilterOperator::NotContains => "!~",
            FilterOperator::MatchRegex => ".*",
            FilterOperator::NotMatchRegex => "!.*",
        }
    }

    pub fn matches(&self, value: &str, pattern: &str) -> bool {
        let pattern_trimmed = pattern.trim();
        if pattern_trimmed.is_empty() {
            return true;
        }

        // Automatic user prefix handling
        if let Some(rest) = pattern_trimmed.strip_prefix("<=") {
            return Self::LessEqual.matches_num(value, rest.trim());
        } else if let Some(rest) = pattern_trimmed.strip_prefix(">=") {
            return Self::GreaterEqual.matches_num(value, rest.trim());
        } else if let Some(rest) = pattern_trimmed.strip_prefix("<") {
            return Self::LessEqual.matches_num(value, rest.trim());
        } else if let Some(rest) = pattern_trimmed.strip_prefix(">") {
            return Self::GreaterEqual.matches_num(value, rest.trim());
        } else if let Some(rest) = pattern_trimmed.strip_prefix("=") {
            return Self::Exact.matches(value, rest.trim());
        }

        match self {
            FilterOperator::Contains => value.to_lowercase().contains(&pattern_trimmed.to_lowercase()),
            FilterOperator::Exact => value.trim_start_matches('#').eq_ignore_ascii_case(pattern_trimmed.trim_start_matches('#')),
            FilterOperator::NotContains => !value.to_lowercase().contains(&pattern_trimmed.to_lowercase()),
            FilterOperator::LessEqual => self.matches_num(value, pattern_trimmed),
            FilterOperator::GreaterEqual => self.matches_num(value, pattern_trimmed),
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

    fn matches_num(&self, value: &str, pattern: &str) -> bool {
        if let (Some(val_num), Some(pat_num)) = (extract_number(value), extract_number(pattern)) {
            match self {
                FilterOperator::LessEqual => val_num <= pat_num,
                FilterOperator::GreaterEqual => val_num >= pat_num,
                _ => false,
            }
        } else {
            value.to_lowercase().contains(&pattern.to_lowercase())
        }
    }
}

fn extract_number(s: &str) -> Option<f32> {
    let clean: String = s.chars().filter(|c| c.is_numeric() || *c == '.' || *c == '-').collect();
    clean.parse::<f32>().ok()
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
