fn is_format_noise(low: &str) -> bool {
    if matches!(
        low,
        "gguf" | "imatrix" | "nvfp4" | "fp8" | "bf16" | "fp16" | "int8" | "int4"
            | "awq" | "gptq" | "exl2" | "mlx" | "onnx" | "instruct" | "chat" | "base"
            | "uncensored" | "abliterated" | "obliterated" | "heretic" | "pruned"
            | "repackaged" | "thinking" | "thinkingcap" | "cyber" | "dm"
    ) {
        return true;
    }
    // Generic quantization prefix tokens like Q4, Q8, IQ4, K, S, M, L
    let tail = low.strip_prefix("iq").or_else(|| low.strip_prefix('q'));
    if let Some(rest) = tail {
        if !rest.is_empty() && rest.chars().next().is_some_and(|c| c.is_ascii_digit()) && rest.len() <= 4 {
            return true;
        }
    }
    false
}

fn is_modifier_word(low: &str) -> bool {
    matches!(
        low,
        "image" | "video" | "audio" | "voice" | "tts" | "speech" | "music" | "3d"
            | "coder" | "code" | "codex" | "chat" | "instruct" | "edit" | "inpaint"
            | "control" | "turbo" | "flash" | "pro" | "max" | "ultra" | "mini"
            | "nano" | "micro" | "tiny" | "small" | "medium" | "large" | "lite"
            | "fast" | "base" | "core" | "next" | "plus" | "omni" | "vision" | "vl"
            | "vlm" | "reason" | "reasoning" | "think" | "thinking" | "distill"
            | "step" | "diffusion" | "flow" | "oss" | "dev" | "schnell" | "klein"
            | "body" | "flex" | "ref" | "multiview" | "multilingual" | "preview" | "experimental"
    )
}

pub fn is_version_or_size_token(tok: &str) -> bool {
    let low = tok.trim().to_lowercase();
    if low.is_empty() {
        return false;
    }
    let trimmed_v = low.strip_prefix('v').unwrap_or(&low);
    if trimmed_v.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return true;
    }
    // Single-letter + version/model marker like "r1", "h3", "k2.5", "v2.6"
    let mut chars = low.chars();
    if let (Some(first), Some(second)) = (chars.next(), chars.next()) {
        if first.is_ascii_alphabetic() && second.is_ascii_digit() {
            return true;
        }
    }
    false
}

fn strip_trailing_version(tok: &str) -> &str {
    let bytes = tok.as_bytes();
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1].is_ascii_digit() || bytes[end - 1] == b'.') {
        end -= 1;
    }
    let prefix = &tok[..end];
    if prefix.chars().filter(|c| c.is_ascii_alphabetic()).count() >= 2 {
        prefix.trim_end_matches('.')
    } else {
        tok
    }
}

pub fn canonicalize_family_case(raw: &str) -> String {
    if raw.is_empty() {
        return "Other".to_string();
    }
    if raw.chars().any(|c| c.is_ascii_uppercase()) {
        return raw.to_string();
    }
    let has_vowel = raw.chars().any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'));
    if raw.len() <= 3 && !has_vowel {
        return raw.to_uppercase();
    }
    let mut chars = raw.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Other".to_string(),
    }
}

pub fn canonicalize_provider_slug(slug: &str) -> String {
    let clean = slug.trim();
    if clean.is_empty() {
        return "Cloud".to_string();
    }
    clean
        .split(['-', '_'])
        .filter(|p| !p.is_empty())
        .map(|part| {
            let low = part.to_lowercase();
            let has_vowel = low.chars().any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'));
            if low == "ai" || (low.len() <= 2 && !has_vowel) {
                low.to_uppercase()
            } else {
                let mut c = low.chars();
                c.next().map(|f| format!("{}{}", f.to_uppercase(), c.as_str())).unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// Automatically extracts an architectural family name purely from repo/model morphology
/// without relying on any hardcoded catalog of known models or vendors.
pub fn extract_dynamic_family(full_id: &str, clean: &str) -> String {
    let org_segs: Vec<String> = full_id
        .split_once('/')
        .map(|(org, _)| {
            org.split(|c: char| !c.is_alphanumeric())
                .map(|s| s.to_lowercase())
                .filter(|s| s.len() >= 3)
                .collect()
        })
        .unwrap_or_default();

    let stem = clean.split('/').last().unwrap_or(clean);
    let tokens: Vec<&str> = stem
        .split(|c: char| matches!(c, '-' | '_' | ' ' | ':' | '(' | ')' | '[' | ']' | '/'))
        .filter(|t| !t.is_empty())
        .collect();

    let mut i = 0;
    while i + 1 < tokens.len() {
        let cur = tokens[i];
        let cur_low = cur.to_lowercase();
        let next = tokens[i + 1];
        let next_low = next.to_lowercase();

        if is_format_noise(&cur_low) || cur.chars().all(|c| c.is_ascii_digit() || c == '.') {
            i += 1;
            continue;
        }
        // Skip wrapper/author segment duplicated from Hugging Face namespace (e.g., huihui-ai/Huihui-...)
        if org_segs.iter().any(|seg| cur_low == *seg)
            && next.len() >= 3
            && !is_version_or_size_token(&next_low)
            && !is_modifier_word(&next_low)
        {
            i += 1;
            continue;
        }
        // Skip short non-acronym lab prefix right before `[Architecture][Version]` (e.g., Meta-Llama-3.1, not Muse-Glimmer-30B)
        if i == 0
            && i + 2 < tokens.len()
            && cur.len() <= 4
            && cur.chars().all(|c| c.is_ascii_alphabetic())
            && !cur.chars().all(|c| c.is_ascii_uppercase())
        {
            let next_base = strip_trailing_version(next);
            let third_low = tokens[i + 2].to_lowercase();
            let third_is_param = (third_low.ends_with('b') || third_low.ends_with('m'))
                && third_low[..third_low.len() - 1].chars().next().is_some_and(|c| c.is_ascii_digit());
            if next_base.len() >= 3
                && next_base.chars().all(|c| c.is_ascii_alphabetic())
                && !is_modifier_word(&next_base.to_lowercase())
                && is_version_or_size_token(tokens[i + 2])
                && !third_is_param
            {
                i += 1;
                continue;
            }
        }
        break;
    }

    let Some(&raw_fam) = tokens.get(i) else {
        return "Other".to_string();
    };
    let stripped = strip_trailing_version(raw_fam);
    if (stripped.len() == 1 || (stripped.len() <= 3 && stripped.chars().any(|c| c.is_ascii_digit())))
        && i + 1 < tokens.len()
    {
        let next_raw = tokens[i + 1];
        let next_stripped = strip_trailing_version(next_raw);
        if !next_stripped.is_empty() && !next_raw.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return format!("{}-{}", canonicalize_family_case(stripped), canonicalize_family_case(next_stripped));
        }
    }
    canonicalize_family_case(stripped)
}
