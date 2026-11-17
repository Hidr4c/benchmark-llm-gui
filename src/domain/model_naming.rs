use crate::domain::modality::ModelModality;
use crate::domain::model::ModelBenchmark;

/// Derives a normalized parent/root family name dynamically from a model variant without hardcoded catalogs.
pub fn compute_root_model_name(model: &ModelBenchmark) -> String {
    let fam = &model.family;

    if model.is_cloud {
        const CLOUD_TIERS: [&str; 14] = [
            " (free)", ":free", " preview", " thinking", " reasoning",
            " turbo", " pro", " mini", " lite", " fast", " high", " medium", " low", " max",
        ];
        let mut base = model.name.trim().to_string();
        loop {
            let low = base.to_lowercase();
            if let Some(suffix) = CLOUD_TIERS.iter().find(|s| low.ends_with(**s)) {
                let keep = base.len().saturating_sub(suffix.len());
                base = base[..keep].trim().to_string();
            } else {
                break;
            }
        }
        return if base.is_empty() { model.name.clone() } else { base };
    }

    // Non-LLM generators (Image, Video, 3D, Audio) & unsized models keep their distinct model identity
    if !matches!(model.modality, ModelModality::Text | ModelModality::Multimodal) || model.param_count_b.is_none() {
        let mut clean = model.name.replace(" (ComfyUI)", "").replace(" (TTS)", "");
        for tag in ["-Q4_K_M", "-Q8_0", "-Q6_K", "-Q5_K_M", "-GGUF", "-Pruned", "-Int8"] {
            clean = clean.replace(tag, "");
        }
        return clean.trim().to_string();
    }

    let p_str = match model.param_count_b {
        Some(p) if p < 1.0 => format!("{:.0}M", p * 1000.0),
        Some(p) if (p.fract() - 0.0).abs() < 0.05 => format!("{:.0}B", p),
        Some(p) => format!("{:.1}B", p),
        None => return model.name.clone(),
    };

    let tokens: Vec<&str> = model.name.split(['-', '_', ' ']).filter(|t| !t.is_empty()).collect();
    let is_param_tok = |t: &str| {
        let low = t.to_lowercase();
        (low.ends_with('b') || low.ends_with('m'))
            && low.len() >= 2
            && low[..low.len() - 1].chars().all(|c| c.is_ascii_digit() || c == '.')
    };
    let end_idx = tokens.iter().position(|t| is_param_tok(t)).unwrap_or(tokens.len());
    let fam_low = fam.to_lowercase();
    let start_idx = if fam != "Other" && !fam_low.is_empty() {
        tokens[..end_idx]
            .iter()
            .position(|t| t.to_lowercase().starts_with(&fam_low))
            .unwrap_or(0)
    } else {
        0
    };

    let mut stem = Vec::new();
    for tok in &tokens[start_idx..end_idx] {
        let low = tok.to_lowercase();
        if matches!(
            low.as_str(),
            "distill" | "abliterated" | "uncensored" | "heretic" | "pruned"
                | "instruct" | "chat" | "it" | "base" | "thinking" | "gguf"
        ) {
            break;
        }
        // Split dotted version attached to family token (e.g. "Qwen3.8" -> "Qwen 3.8")
        if stem.is_empty() && tok.contains('.') {
            if let Some(pos) = tok.find(|c: char| c.is_ascii_digit()) {
                if pos >= 2 {
                    stem.push(tok[..pos].to_string());
                    stem.push(tok[pos..].to_string());
                    continue;
                }
            }
        }
        stem.push((*tok).to_string());
        if stem.len() >= 2 {
            break;
        }
    }

    if !stem.is_empty() {
        format!("{} {p_str}", stem.join(" "))
    } else if fam != "Other" {
        format!("{fam} {p_str}")
    } else {
        model.name.clone()
    }
}
