use serde::{Deserialize, Serialize};
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::domain::table_filter::FilterOperator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableColumnId {
    Rank,
    Model,
    Modality,
    Year,
    TargetHardware,
    Size,
    Efficiency,
    Score,
    Confidence,
    RealWorld,
    Speed,
    Popularity,
    MmluPro,
    SweBench,
    Agentic,
    OpenLlm,
    ImageBench,
    VideoBench,
    AudioBench,
    Math,
    Gpqa,
    Ifeval,
    CloudPrice,
}

impl TableColumnId {
    pub fn all() -> &'static [TableColumnId] {
        &[
            TableColumnId::Rank,
            TableColumnId::Model,
            TableColumnId::Modality,
            TableColumnId::Year,
            TableColumnId::TargetHardware,
            TableColumnId::Size,
            TableColumnId::Efficiency,
            TableColumnId::Score,
            TableColumnId::Confidence,
            TableColumnId::RealWorld,
            TableColumnId::Speed,
            TableColumnId::Popularity,
            TableColumnId::MmluPro,
            TableColumnId::SweBench,
            TableColumnId::Agentic,
            TableColumnId::OpenLlm,
            TableColumnId::ImageBench,
            TableColumnId::VideoBench,
            TableColumnId::AudioBench,
            TableColumnId::Math,
            TableColumnId::Gpqa,
            TableColumnId::Ifeval,
            TableColumnId::CloudPrice,
        ]
    }

    pub fn default_order() -> Vec<TableColumnId> {
        Self::all().to_vec()
    }

    pub fn default_operator(&self) -> FilterOperator {
        FilterOperator::Contains
    }

    pub fn placeholder(&self) -> &'static str {
        "..."
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
                | TableColumnId::ImageBench
                | TableColumnId::VideoBench
                | TableColumnId::AudioBench
                | TableColumnId::Math
                | TableColumnId::Gpqa
                | TableColumnId::Ifeval
        )
    }

    pub fn default_sort_field(&self) -> Option<(SortField, SortOrder)> {
        match self {
            TableColumnId::Rank => None,
            TableColumnId::Model => Some((SortField::Name, SortOrder::Ascending)),
            TableColumnId::Modality => Some((SortField::Modality, SortOrder::Ascending)),
            TableColumnId::Year => Some((SortField::Year, SortOrder::Descending)),
            TableColumnId::TargetHardware => Some((SortField::Memory, SortOrder::Ascending)),
            TableColumnId::Size => Some((SortField::Size, SortOrder::Ascending)),
            TableColumnId::Efficiency => Some((SortField::Efficiency, SortOrder::Descending)),
            TableColumnId::Score => Some((SortField::Intelligence, SortOrder::Descending)),
            TableColumnId::Confidence => Some((SortField::Confidence, SortOrder::Descending)),
            TableColumnId::RealWorld => Some((SortField::RealWorld, SortOrder::Descending)),
            TableColumnId::Speed => Some((SortField::Speed, SortOrder::Descending)),
            TableColumnId::Popularity => Some((SortField::Downloads, SortOrder::Descending)),
            TableColumnId::MmluPro => None,
            TableColumnId::SweBench => Some((SortField::Coding, SortOrder::Descending)),
            TableColumnId::Agentic => Some((SortField::Agentic, SortOrder::Descending)),
            TableColumnId::OpenLlm => Some((SortField::Intelligence, SortOrder::Descending)),
            TableColumnId::ImageBench => Some((SortField::ImageBench, SortOrder::Descending)),
            TableColumnId::VideoBench => Some((SortField::VideoBench, SortOrder::Descending)),
            TableColumnId::AudioBench => Some((SortField::AudioBench, SortOrder::Descending)),
            TableColumnId::Math => None,
            TableColumnId::Gpqa => None,
            TableColumnId::Ifeval => None,
            TableColumnId::CloudPrice => Some((SortField::Price, SortOrder::Ascending)),
        }
    }

    pub fn extract_value(&self, model: &ModelBenchmark, rank_str: &str) -> String {
        match self {
            TableColumnId::Rank => rank_str.to_string(),
            TableColumnId::Model => model.name.clone(),
            TableColumnId::Modality => {
                let mut text = format!("{} {}", model.modality.tag_short(), model.modality.label());
                if model.specialty != crate::domain::specialty::DomainSpecialty::General {
                    text.push(' ');
                    text.push_str(model.specialty.badge_text());
                    text.push(' ');
                    text.push_str(model.specialty.label_fr());
                }
                text
            }
            TableColumnId::Year => model.release_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into()),
            TableColumnId::TargetHardware => target_hardware_text(model).to_string(),
            TableColumnId::Size => {
                if model.is_cloud { "Cloud API".into() } else { format!("{:.1} Go", model.size_gb) }
            }
            TableColumnId::Efficiency => format!("{:.1}", model.efficiency_ratio()),
            TableColumnId::Score => model.intelligence().map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Confidence => {
                match model.score_breakdown(&crate::domain::model::SourceSelection::default()).map(|b| b.confidence) {
                    Some(crate::domain::scoring::ScoreConfidence::High) => "[OK] Haute High 3+".into(),
                    Some(crate::domain::scoring::ScoreConfidence::Medium) => "[~] Moyenne Medium 2+".into(),
                    Some(crate::domain::scoring::ScoreConfidence::Low) | None => "[!] Faible Low 1".into(),
                }
            }
            TableColumnId::RealWorld => model.economic.as_ref().and_then(|e| e.composite_score())
                .or(model.scores.real_world)
                .map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Speed => model.tokens_per_sec.map(|s| format!("{s:.0}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Popularity => model.downloads.to_string(),
            TableColumnId::MmluPro => model.scores.mmlu_pro.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::SweBench => model.scores.coding.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Agentic => model.scores.agentic.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::OpenLlm => model.scores.open_llm.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::ImageBench => model.scores.image_bench.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::VideoBench => model.scores.video_bench.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::AudioBench => model.scores.audio_bench.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Math => model.scores.math_lvl5.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Gpqa => model.scores.gpqa.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::Ifeval => model.scores.ifeval.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            TableColumnId::CloudPrice => model.pricing_output_usd.or(model.pricing_input_usd)
                .filter(|&p| p > 0.0)
                .map(|p| format!("${p:.3}"))
                .unwrap_or_else(|| if model.is_cloud { "N/A (API)".into() } else { "Local (0.00€)".into() }),
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
