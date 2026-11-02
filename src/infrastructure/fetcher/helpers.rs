use crate::domain::model::BenchmarkScores;

pub fn parse_model_identity(full_id: &str) -> (String, String, Option<f32>) {
    let repo_name = full_id.split('/').last().unwrap_or(full_id);
    let mut clean = repo_name.to_string();

    let strip_suffixes = [
        "-GGUF", "-gguf", ".gguf", "-iMatrix", "-NVFP4", "-GSQ-RCO",
        "-abliterated", "-MTP", "-UD", "-unsloth", "-Instruct", "-instruct"
    ];

    for s in strip_suffixes {
        clean = clean.replace(s, "");
    }

    let lower = clean.to_lowercase();
    let family = if lower.contains("deepseek") {
        "DeepSeek".to_string()
    } else if lower.contains("qwen") {
        "Qwen".to_string()
    } else if lower.contains("llama") {
        "Llama".to_string()
    } else if lower.contains("mistral") || lower.contains("mixtral") || lower.contains("ministral") {
        "Mistral".to_string()
    } else if lower.contains("smollm") {
        "SmolLM".to_string()
    } else if lower.contains("gemma") {
        "Gemma".to_string()
    } else if lower.contains("phi") {
        "Phi".to_string()
    } else if lower.contains("danube") {
        "Danube".to_string()
    } else if lower.contains("tinyllama") {
        "TinyLlama".to_string()
    } else {
        "Other".to_string()
    };

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
    let base = match family {
        "Qwen" => "qwen2.5",
        "Llama" => "llama3.2",
        "SmolLM" => "smollm2",
        "Mistral" => "mistral",
        "DeepSeek" => "deepseek-r1",
        "Gemma" => "gemma2",
        "Phi" => "phi4",
        _ => return None,
    };

    if let Some(p) = params_b {
        if p >= 70.0 {
            Some(format!("{base}:70b"))
        } else if p >= 30.0 {
            Some(format!("{base}:32b"))
        } else if p >= 14.0 {
            Some(format!("{base}:14b"))
        } else if p >= 7.0 {
            Some(format!("{base}:8b"))
        } else if p >= 3.0 {
            Some(format!("{base}:3b"))
        } else if p >= 1.0 {
            Some(format!("{base}:1.5b"))
        } else if p >= 0.3 {
            Some(format!("{base}:0.5b"))
        } else {
            Some(format!("{base}:latest"))
        }
    } else {
        Some(format!("{base}:latest"))
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
    let mut bonus: f32 = 0.0;

    // 2026 native architectures
    if lower.contains("qwen3") || lower.contains("qwen-3") || lower.contains("qwen3.8") {
        bonus += 5.0;
    } else if lower.contains("qwen2.5") || lower.contains("llama3.3") || lower.contains("gemma-2") {
        bonus += 2.0;
    }

    // Full frontier reasoning models
    if (lower.contains("deepseek-r1") || lower.contains("deepseek")) && !is_distill {
        bonus += 4.0;
    }

    // Moderate distillation penalty for compressed student models
    if is_distill {
        bonus -= 3.5;
    }

    let mmlu = (base_intel + bonus).clamp(10.0, 96.0);
    let is_coder = lower.contains("coder") || lower.contains("code");
    let is_agent = lower.contains("r1") || lower.contains("reason") || lower.contains("agent") || lower.contains("think") || lower.contains("tool");

    let coding_score = if is_coder { (mmlu * 1.05 + 5.0).min(96.0) } else { (mmlu * 0.88).max(10.0) };
    let agentic_score = if is_agent { (mmlu * 1.02 + 3.0).min(96.0) } else { (mmlu * 0.90).max(10.0) };

    BenchmarkScores {
        open_llm: Some(mmlu),
        ifeval: Some((mmlu + 4.0).min(99.0)),
        bbh: Some((mmlu * 0.95).round()),
        math_lvl5: Some((mmlu * 0.75).round()),
        gpqa: Some((mmlu * 0.55).round()),
        mmlu_pro: Some(mmlu),
        musr: Some((mmlu * 0.5).round()),
        coding: Some((coding_score * 10.0).round() / 10.0),
        agentic: Some((agentic_score * 10.0).round() / 10.0),
        average: Some(((mmlu + coding_score + agentic_score) / 3.0 * 10.0).round() / 10.0),
        real_world: None,
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

pub fn extract_release_year(id: &str, name: &str, created_at: Option<&str>) -> Option<u16> {
    let combined = format!("{id} {name}").to_lowercase();

    // 1. Explicit generational architectures (2026 frontier takes precedence over upload metadata)
    if combined.contains("2026")
        || combined.contains("gemini-4")
        || combined.contains("argon")
        || combined.contains("claude-4")
        || combined.contains("claude-5")
        || combined.contains("qwen3")
        || combined.contains("qwen-3")
        || combined.contains("gpt-6")
        || combined.contains("deepseek-v4")
        || combined.contains("mistral-small-4")
        || combined.contains("mistral-large-4")
    {
        return Some(2026);
    }

    // 2. Repository upload timestamp if available
    if let Some(c) = created_at {
        if c.len() >= 4 {
            if let Ok(y) = c[0..4].parse::<u16>() {
                if (2020..=2030).contains(&y) {
                    return Some(y);
                }
            }
        }
    }

    // 3. Fallback heuristic detection
    if combined.contains("2025")
        || combined.contains("2501")
        || combined.contains("2502")
        || combined.contains("r1")
        || combined.contains("deepseek-v3")
        || combined.contains("gpt-5")
        || combined.contains("o3")
    {
        Some(2025)
    } else if combined.contains("2024")
        || combined.contains("2411")
        || combined.contains("llama-3")
        || combined.contains("smollm2")
        || combined.contains("gemma-2")
        || combined.contains("qwen2.5")
    {
        Some(2024)
    } else if combined.contains("2023")
        || combined.contains("llama-2")
        || combined.contains("mistral-7b-v0.1")
    {
        Some(2023)
    } else {
        Some(2025)
    }
}
