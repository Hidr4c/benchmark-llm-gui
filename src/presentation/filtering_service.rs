use crate::domain::hardware::HardwarePlatform;
use crate::domain::license::LicenseCategory;
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::filtering::{
    ConfidenceFilter, FilterState, ModalityFilter, ModelScope, RootModelGroup, SpecialtyFilter, TableColumnMode,
};

fn cmp_f32(a: f32, b: f32, order: SortOrder) -> std::cmp::Ordering {
    let sa = if a.is_finite() { a } else { 0.0 };
    let sb = if b.is_finite() { b } else { 0.0 };
    match order {
        SortOrder::Descending => sb.total_cmp(&sa),
        SortOrder::Ascending => sa.total_cmp(&sb),
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
            if !filters.search_text.trim().is_empty() {
                let q = filters.search_text.trim().to_lowercase();
                let matches_name = m.name.to_lowercase().contains(&q);
                let matches_id = m.id.to_lowercase().contains(&q);
                let matches_fam = m.family.to_lowercase().contains(&q);
                let matches_root = m.root_model_name().to_lowercase().contains(&q);
                if !matches_name && !matches_id && !matches_fam && !matches_root {
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

            if let SpecialtyFilter::Specific(target_spec) = filters.specialty {
                if !target_spec.matches_model(m) {
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

            if filters.selected_family != "All" && filters.selected_family != "Tous" && !m.family.eq_ignore_ascii_case(&filters.selected_family) {
                return None;
            }

            match filters.confidence_level {
                ConfidenceFilter::All => {},
                ConfidenceFilter::MediumPlus => {
                    if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                        if breakdown.confidence == crate::domain::scoring::ScoreConfidence::Low { return None; }
                    } else { return None; }
                }
                ConfidenceFilter::HighOnly => {
                    if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                        if breakdown.confidence != crate::domain::scoring::ScoreConfidence::High { return None; }
                    } else { return None; }
                }
            }
            if filters.column_mode == TableColumnMode::StrictVerified {
                if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                    if breakdown.confidence == crate::domain::scoring::ScoreConfidence::Low { return None; }
                } else { return None; }
            }

            if !filters.table_prefs.matches_model(m, "") {
                return None;
            }

            Some(idx)
        }).collect();

        let order = filters.sort_order;

        let tie = |a: usize, b: usize| {
            models[b].downloads
                .cmp(&models[a].downloads)
                .then_with(|| models[a].name.to_lowercase().cmp(&models[b].name.to_lowercase()))
                .then_with(|| models[a].id.cmp(&models[b].id))
        };

        match filters.sort_field {
            SortField::Intelligence => indices.sort_by(|&a, &b| cmp_f32(models[a].score_for_sources(&filters.sources).unwrap_or(0.0), models[b].score_for_sources(&filters.sources).unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::Efficiency => indices.sort_by(|&a, &b| cmp_f32(models[a].efficiency_ratio_with_sources(&filters.sources), models[b].efficiency_ratio_with_sources(&filters.sources), order).then_with(|| tie(a, b))),
            SortField::Speed => indices.sort_by(|&a, &b| cmp_f32(models[a].tokens_per_sec.unwrap_or(0.0), models[b].tokens_per_sec.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::Size => indices.sort_by(|&a, &b| cmp_f32(models[a].size_gb, models[b].size_gb, order).then_with(|| tie(a, b))),
            SortField::Memory => indices.sort_by(|&a, &b| cmp_f32(models[a].vram_rec_gb, models[b].vram_rec_gb, order).then_with(|| tie(a, b))),
            SortField::Downloads => indices.sort_by(|&a, &b| cmp_ord(models[a].downloads, models[b].downloads, order).then_with(|| models[a].id.cmp(&models[b].id))),
            SortField::Name => indices.sort_by(|&a, &b| cmp_ord(models[a].name.to_lowercase(), models[b].name.to_lowercase(), order).then_with(|| models[a].id.cmp(&models[b].id))),
            SortField::Price => indices.sort_by(|&a, &b| cmp_f32(models[a].pricing_input_usd.unwrap_or(0.0), models[b].pricing_input_usd.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::Year => indices.sort_by(|&a, &b| cmp_ord(models[a].release_year.unwrap_or(0), models[b].release_year.unwrap_or(0), order).then_with(|| tie(a, b))),
            SortField::RealWorld => indices.sort_by(|&a, &b| cmp_f32(
                models[a].economic.as_ref().and_then(|e| e.composite_score()).or(models[a].scores.real_world).unwrap_or(0.0),
                models[b].economic.as_ref().and_then(|e| e.composite_score()).or(models[b].scores.real_world).unwrap_or(0.0),
                order
            ).then_with(|| tie(a, b))),
            SortField::Coding => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.coding.unwrap_or(0.0), models[b].scores.coding.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::Agentic => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.agentic.unwrap_or(0.0), models[b].scores.agentic.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::ImageBench => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.image_bench.unwrap_or(0.0), models[b].scores.image_bench.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::VideoBench => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.video_bench.unwrap_or(0.0), models[b].scores.video_bench.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::AudioBench => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.audio_bench.unwrap_or(0.0), models[b].scores.audio_bench.unwrap_or(0.0), order).then_with(|| tie(a, b))),
            SortField::Confidence => indices.sort_by(|&a, &b| {
                let rank = |m: &ModelBenchmark| match m.score_breakdown(&filters.sources).map(|bk| bk.confidence) {
                    Some(crate::domain::scoring::ScoreConfidence::High) => 3u8,
                    Some(crate::domain::scoring::ScoreConfidence::Medium) => 2u8,
                    Some(crate::domain::scoring::ScoreConfidence::Low) => 1u8,
                    None => 0u8,
                };
                cmp_ord(rank(&models[a]), rank(&models[b]), order)
                    .then_with(|| cmp_f32(models[a].score_for_sources(&filters.sources).unwrap_or(0.0), models[b].score_for_sources(&filters.sources).unwrap_or(0.0), SortOrder::Descending))
                    .then_with(|| tie(a, b))
            }),
            SortField::Modality => indices.sort_by(|&a, &b| {
                cmp_ord(models[a].modality.tag_short(), models[b].modality.tag_short(), order)
                    .then_with(|| cmp_f32(models[a].score_for_sources(&filters.sources).unwrap_or(0.0), models[b].score_for_sources(&filters.sources).unwrap_or(0.0), SortOrder::Descending))
                    .then_with(|| tie(a, b))
            }),
        }

        if let Some(rank_filter) = filters.table_prefs.column_filters.get(&crate::domain::table_preferences::TableColumnId::Rank) {
            if rank_filter.active && !rank_filter.pattern.trim().is_empty() {
                let mut filtered_by_rank = Vec::new();
                for (pos, idx) in indices.into_iter().enumerate() {
                    let rank_num = (pos + 1).to_string();
                    let rank_str = format!("#{rank_num}");
                    if rank_filter.matches(&rank_num) || rank_filter.matches(&rank_str) {
                        filtered_by_rank.push(idx);
                    }
                }
                indices = filtered_by_rank;
            }
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

    /// Collects sorted canonical family names dynamically from the loaded catalog.
    pub fn extract_unique_families(models: &[ModelBenchmark]) -> Vec<String> {
        let mut by_key: std::collections::BTreeMap<String, (String, usize)> = std::collections::BTreeMap::new();
        let mut has_other = false;
        for m in models {
            let trimmed = m.family.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.to_lowercase();
            if key == "other" {
                has_other = true;
                continue;
            }
            by_key
                .entry(key)
                .and_modify(|(best, count)| {
                    *count += 1;
                    let cur_upper = trimmed.chars().filter(|c| c.is_ascii_uppercase()).count();
                    let best_upper = best.chars().filter(|c| c.is_ascii_uppercase()).count();
                    if cur_upper > best_upper {
                        *best = trimmed.to_string();
                    }
                })
                .or_insert_with(|| (trimmed.to_string(), 1));
        }
        let mut out: Vec<String> = by_key.into_values().map(|(label, _)| label).collect();
        out.sort_by_key(|s| s.to_lowercase());
        if has_other {
            out.push("Other".to_string());
        }
        out
    }
}
