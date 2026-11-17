use crate::domain::model::BenchmarkScores;
pub use super::family_parser::{
    canonicalize_family_case, canonicalize_provider_slug, extract_dynamic_family, is_version_or_size_token,
};

pub fn parse_model_identity(full_id: &str) -> (String, String, Option<f32>) {
    let repo_name = full_id.split('/').last().unwrap_or(full_id);
    let mut clean = repo_name.to_string();

    let strip_suffixes = [
        "-GGUF", "-gguf", ".gguf", "-iMatrix", "-NVFP4", "-GSQ-RCO",
        "-abliterated", "-MTP", "-UD", "-unsloth", "-Instruct", "-instruct",
    ];

    for s in strip_suffixes {
        clean = clean.replace(s, "");
    }

    let family = extract_dynamic_family(full_id, &clean);

    let mut params_b = None;
    let tokens: Vec<&str> = clean.split(|c: char| !c.is_alphanumeric() && c != '.').collect();
    for token in tokens {
        let t_low = token.to_lowercase();
        if t_low.ends_with('b') && t_low.len() >= 2 {
            let num_str = &t_low[..t_low.len() - 1];
            if let Ok(num) = num_str.parse::<f32>() {
                if (0.05..=1000.0).contains(&num) {
                    params_b = Some(num);
                    break;
                }
            }
        } else if t_low.ends_with('m') && t_low.len() >= 2 {
            let num_str = &t_low[..t_low.len() - 1];
            if let Ok(num) = num_str.parse::<f32>() {
                if (10.0..=1000.0).contains(&num) {
                    params_b = Some(num / 1000.0);
                    break;
                }
            }
        }
    }

    (clean, family, params_b)
}

pub fn estimate_gguf_size(params_b: Option<f32>, name: &str) -> f32 {
    if let Some(p) = params_b {
        let mut est = p * 0.58;
        if est < 0.15 {
            est = 0.15;
        }
        (est * 10.0).round() / 10.0
    } else {
        let lower = name.to_lowercase();
        if lower.contains("70b") {
            40.5
        } else if lower.contains("32b") || lower.contains("35b") {
            19.5
        } else if lower.contains("27b") {
            16.2
        } else if lower.contains("14b") {
            8.6
        } else if lower.contains("8b") || lower.contains("9b") || lower.contains("7b") {
            4.9
        } else if lower.contains("3b") || lower.contains("4b") {
            2.2
        } else if lower.contains("1.5b") || lower.contains("1b") {
            0.9
        } else if lower.contains("0.5b") {
            0.35
        } else if lower.contains("360m") || lower.contains("135m") {
            0.22
        } else {
            3.5
        }
    }
}

pub fn normalize(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

pub fn generate_ollama_tag(family: &str, params_b: Option<f32>) -> Option<String> {
    let base = family.trim();
    if base.is_empty() || base.eq_ignore_ascii_case("other") {
        return None;
    }
    let slug = base.to_lowercase().replace(' ', "-");
    let p = params_b?;
    if p >= 70.0 {
        Some(format!("{slug}:70b"))
    } else if p >= 30.0 {
        Some(format!("{slug}:32b"))
    } else if p >= 14.0 {
        Some(format!("{slug}:14b"))
    } else if p >= 7.0 {
        Some(format!("{slug}:8b"))
    } else if p >= 3.0 {
        Some(format!("{slug}:3b"))
    } else if p >= 1.0 {
        Some(format!("{slug}:1.5b"))
    } else if p >= 0.3 {
        Some(format!("{slug}:0.5b"))
    } else {
        Some(format!("{slug}:latest"))
    }
}

pub fn estimate_scores(name: &str, params_b: Option<f32>) -> BenchmarkScores {
    let lower = name.to_lowercase();
    let base_intel: f32 = match params_b {
        Some(p) if p >= 65.0 => 83.0,
        Some(p) if p >= 30.0 => 78.0,
        Some(p) if p >= 20.0 => 73.0,
        Some(p) if p >= 12.0 => 65.0,
        Some(p) if p >= 7.0 => 55.0,
        Some(p) if p >= 3.0 => 43.0,
        Some(p) if p >= 1.0 => 33.0,
        Some(p) if p >= 0.4 => 23.0,
        _ => 16.0,
    };

    let is_distill = lower.contains("distill");
    let is_reasoning = lower.contains("-r1") || lower.contains("reason") || lower.contains("think");
    let mut bonus: f32 = if is_reasoning && !is_distill { 3.5 } else { 0.0 };
    if is_distill {
        bonus -= 3.5;
    }

    let mmlu = (base_intel + bonus).clamp(10.0, 96.0);

    // Heuristic size fallback MUST NOT fabricate verified SWE-Bench (coding) or BFCL (agentic) scores
    BenchmarkScores {
        open_llm: Some(mmlu),
        average: Some(mmlu),
        ..Default::default()
    }
}

pub fn estimate_tps(params_b: Option<f32>) -> (Option<f32>, Option<String>) {
    match params_b {
        Some(p) if p <= 0.5 => (Some(280.0), Some("Micro-IoT / Pi5 CPU".to_string())),
        Some(p) if p <= 1.5 => (Some(180.0), Some("Raspberry Pi 5 CPU".to_string())),
        Some(p) if p <= 4.0 => (Some(120.0), Some("Jetson / RTX 4060".to_string())),
        Some(p) if p <= 8.0 => (Some(95.0), Some("RTX 4060 (8GB)".to_string())),
        Some(p) if p <= 14.0 => (Some(68.0), Some("RTX 4080 (16GB)".to_string())),
        Some(p) if p <= 27.0 => (Some(48.0), Some("RTX 4090 (24GB)".to_string())),
        Some(p) if p <= 35.0 => (Some(38.0), Some("RTX 4090 (Offload)".to_string())),
        _ => (Some(18.0), Some("Workstation / Offload".to_string())),
    }
}

/// Extracts a release year dynamically from repository tokens (`YYYY` or `YYMM`) and API upload date.
pub fn extract_release_year(id: &str, name: &str, created_at: Option<&str>) -> Option<u16> {
    let combined = format!("{id} {name}");
    for tok in combined.split(|c: char| !c.is_ascii_digit()).filter(|t| t.len() == 4) {
        if let Ok(num) = tok.parse::<u16>() {
            if (2022..=2030).contains(&num) {
                return Some(num);
            }
            let yy = num / 100;
            let mm = num % 100;
            if (22..=30).contains(&yy) && (1..=12).contains(&mm) {
                return Some(2000 + yy);
            }
        }
    }

    if let Some(c) = created_at.and_then(|s| s.get(0..4)).and_then(|s| s.parse::<u16>().ok()) {
        if (2020..=2030).contains(&c) {
            return Some(c);
        }
    }

    Some(2025)
}
