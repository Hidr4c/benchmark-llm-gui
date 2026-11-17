use std::collections::{HashMap, HashSet};
use crate::domain::runtime::LocalRuntimeProfile;
use super::awesome_discovery::{
    enrich_repos_via_ungh, parse_awesome_markdown, search_github_topic_repos,
};

const SPEED_CSV_URL: &str = "https://huggingface.co/datasets/llmspeed/llm-speed-benchmarks/raw/main/runs.csv";
const RUNTIME_FEEDS: &[&str] = &[
    "https://raw.githubusercontent.com/shvyac/awesome-local-llm/main/README.md",
    "https://raw.githubusercontent.com/rafska/awesome-local-llm/main/README.md",
    "https://raw.githubusercontent.com/janhq/awesome-local-ai/main/README.md",
];

pub struct RuntimeAdapter {
    client: reqwest::Client,
}

impl RuntimeAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Dynamically discovers local inference software from live community catalogs, GitHub topic search & HF speed runs.
    pub async fn fetch_runtimes(&self) -> Result<Vec<LocalRuntimeProfile>, String> {
        let (speed_stats, md_docs, gguf_search) = tokio::join!(
            self.fetch_backend_speed_stats(),
            self.fetch_markdown_feeds(),
            search_github_topic_repos(&self.client, "topic:gguf+stars:>1000", 25),
        );

        let allowed = [
            "inference engines", "inference platforms", "serving frameworks",
            "apple silicon", "edge", "desktop apps", "chat uis", "local-first", "user interfaces",
        ];
        let denied = ["archived", "tutorials", "communities", "fine-tuning", "agent", "rag", "coding agents"];

        let mut ordered_candidates = Vec::new();
        let mut seen_slugs = HashSet::new();
        for item in md_docs.into_iter().flat_map(|md| parse_awesome_markdown(&md, &allowed, &denied)).chain(gguf_search) {
            let desc_low = item.description.to_lowercase();
            let is_non_runtime = desc_low.contains("security")
                || desc_low.contains("adversarial")
                || desc_low.contains("fine-tuning")
                || desc_low.contains("pretrain");
            let key = item.slug.to_lowercase();
            if !is_non_runtime && seen_slugs.insert(key) {
                ordered_candidates.push(item);
            }
        }

        // Preserve deterministic feed priority before capping discovered software entries
        ordered_candidates.truncate(80);
        let candidates = ordered_candidates;

        let slugs: Vec<String> = candidates.iter().map(|c| c.slug.clone()).collect();
        let meta_map = enrich_repos_via_ungh(&self.client, &slugs).await;

        let mut seen_names = HashSet::new();
        let mut profiles = Vec::with_capacity(candidates.len());
        for c in candidates {
            let meta = meta_map.get(&c.slug.to_lowercase());
            let stars = meta.map(|m| m.stars).filter(|&s| s > 0).unwrap_or(c.inline_stars);
            let pushed = meta.map(|m| m.pushed_at.clone()).filter(|s| !s.is_empty()).unwrap_or(c.pushed_at.clone());
            let desc = meta.and_then(|m| m.description.clone()).filter(|d| !d.is_empty()).unwrap_or(c.description.clone());
            let license = meta.and_then(|m| m.license.clone()).or(c.license.clone()).unwrap_or_else(|| "Open-Source".into());
            let lang = meta.and_then(|m| m.language.clone()).or(c.language.clone()).unwrap_or_else(|| "Open-Source".into());

            let blob = format!("{} {} {} {} {}", c.name, c.slug, c.section, desc, c.topics.join(" ")).to_lowercase();
            let (gpu, apple, edge, mobile) = infer_capabilities(&blob);
            let backends = infer_backends(&blob);
            let platforms = infer_platforms(&blob, apple, mobile);
            let stat = match_speed_stat(&c.name, &c.slug, &speed_stats);

            let display_name = clean_runtime_display_name(&c.name, &c.slug);
            if !seen_names.insert(display_name.to_lowercase()) {
                continue;
            }

            profiles.push(LocalRuntimeProfile {
                name: display_name,
                repo_slug: c.slug.clone(),
                category: classify_runtime_category(&c.section, &blob),
                official_url: if c.url.is_empty() { format!("https://github.com/{}", c.slug) } else { c.url },
                stars,
                latest_version: if pushed.is_empty() { "Active".into() } else { format!("Active ({})", pushed.chars().take(7).collect::<String>()) },
                last_updated: if pushed.is_empty() { "-".into() } else { pushed.chars().take(10).collect() },
                license,
                language: lang.clone(),
                platforms,
                backends,
                supports_desktop_gpu: gpu,
                supports_apple_silicon: apple,
                supports_cpu_edge: edge,
                supports_off_grid_mobile: mobile,
                measured_avg_tps: stat.map(|x| x.0),
                measured_runs: stat.map(|x| x.1).unwrap_or(0),
                sample_hardware: stat.map(|x| x.2.clone()).unwrap_or_default(),
                install_cmd: derive_install_hint(&c.slug, &lang),
                description: if desc.is_empty() { format!("{} ({})", c.section, c.slug) } else { desc },
            });
        }

        // Also surface any backend found dynamically in HuggingFace speed CSV not yet represented above
        for (bk, (tps, runs, hw)) in &speed_stats {
            let already = profiles.iter().any(|p| norm_key(&p.name).contains(&norm_key(bk)));
            if !already && *runs >= 3 {
                profiles.push(LocalRuntimeProfile {
                    name: bk.clone(),
                    repo_slug: String::new(),
                    category: "Measured Inference Backend".into(),
                    official_url: SPEED_CSV_URL.into(),
                    stars: 0,
                    latest_version: "Measured".into(),
                    last_updated: "-".into(),
                    license: "Open-Source".into(),
                    language: "-".into(),
                    platforms: vec!["Cross-Platform".into()],
                    backends: vec![bk.clone()],
                    supports_desktop_gpu: true,
                    supports_apple_silicon: bk.to_lowercase().contains("mlx"),
                    supports_cpu_edge: !bk.to_lowercase().contains("vllm"),
                    supports_off_grid_mobile: false,
                    measured_avg_tps: Some(*tps),
                    measured_runs: *runs,
                    sample_hardware: hw.clone(),
                    install_cmd: format!("# {bk} runtime"),
                    description: format!("Discovered dynamically via live speed runs ({runs} samples on {hw})."),
                });
            }
        }

        profiles.sort_by(|a, b| {
            b.rating_score().total_cmp(&a.rating_score())
                .then_with(|| b.stars.cmp(&a.stars))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(profiles)
    }

    async fn fetch_markdown_feeds(&self) -> Vec<String> {
        let mut set = tokio::task::JoinSet::new();
        for (idx, &url) in RUNTIME_FEEDS.iter().enumerate() {
            let client = self.client.clone();
            set.spawn(async move {
                let txt = client.get(url).timeout(std::time::Duration::from_secs(10)).send().await.ok()?.text().await.ok()?;
                Some((idx, txt))
            });
        }
        let mut docs: Vec<Option<String>> = vec![None; RUNTIME_FEEDS.len()];
        while let Some(Ok(Some((idx, txt)))) = set.join_next().await {
            if idx < docs.len() { docs[idx] = Some(txt); }
        }
        docs.into_iter().flatten().collect()
    }

    async fn fetch_backend_speed_stats(&self) -> HashMap<String, (f32, usize, String)> {
        let mut groups: HashMap<String, Vec<(f32, String)>> = HashMap::new();
        let Ok(resp) = self.client.get(SPEED_CSV_URL).timeout(std::time::Duration::from_secs(10)).send().await else {
            return HashMap::new();
        };
        let Ok(text) = resp.text().await else { return HashMap::new(); };
        let mut rdr = csv::ReaderBuilder::new().has_headers(true).flexible(true).from_reader(text.as_bytes());
        for rec in rdr.records().flatten() {
            if rec.len() >= 8 {
                let backend = rec[4].trim().to_lowercase();
                let hw = rec[6].split('+').next().unwrap_or(&rec[6]).trim().to_string();
                if let Ok(tps) = rec[7].parse::<f32>() {
                    if tps > 0.5 && tps < 600.0 && !backend.is_empty() && backend != "top_backend" {
                        groups.entry(backend).or_default().push((tps, hw));
                    }
                }
            }
        }
        groups.into_iter().map(|(k, mut vals)| {
            vals.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mid = &vals[vals.len() / 2];
            (k, ((mid.0 * 10.0).round() / 10.0, vals.len(), mid.1.clone()))
        }).collect()
    }
}

fn clean_runtime_display_name(raw_name: &str, slug: &str) -> String {
    let trimmed = raw_name.trim();
    if trimmed.len() >= 2 && trimmed.len() <= 38 {
        trimmed.to_string()
    } else {
        slug.split('/').last().unwrap_or(trimmed).to_string()
    }
}

fn classify_runtime_category(section: &str, blob: &str) -> String {
    let sec = section.to_lowercase();
    if sec.contains("serving") || blob.contains("serving engine") || blob.contains("pagedattention") {
        "High-Throughput Server".into()
    } else if blob.contains("android") || blob.contains("ios") || blob.contains("smartphone") || blob.contains("off-grid") {
        "Mobile & Edge Runtime".into()
    } else if sec.contains("apple") || blob.contains("apple silicon") || blob.contains("mlx") {
        "Apple Silicon / Edge Engine".into()
    } else if sec.contains("desktop") || sec.contains("chat") || sec.contains("interface") {
        "Desktop GUI / Local Chat".into()
    } else {
        "Core Inference Engine".into()
    }
}

fn infer_capabilities(blob: &str) -> (bool, bool, bool, bool) {
    let mobile = blob.contains("android") || blob.contains("ios") || blob.contains("smartphone")
        || blob.contains("on-device") || blob.contains("off-grid") || blob.contains("air-gap")
        || blob.contains("executorch") || blob.contains("litert");
    let is_core_runner = blob.contains("inference engines") || blob.contains("desktop apps");
    let apple = is_core_runner || blob.contains("metal") || blob.contains("mlx") || blob.contains("macos")
        || blob.contains("apple silicon") || blob.contains("gguf") || blob.contains("ggml");
    let edge = mobile || is_core_runner || blob.contains("cpu") || blob.contains("edge") || blob.contains("arm")
        || blob.contains("raspberry") || blob.contains("gguf") || blob.contains("ggml") || blob.contains("bitnet");
    let gpu = !mobile || blob.contains("cuda") || blob.contains("vulkan") || blob.contains("gpu");
    (gpu, apple, edge, mobile)
}

fn infer_backends(blob: &str) -> Vec<String> {
    let mut b = Vec::new();
    for (kw, tag) in [
        ("cuda", "CUDA"), ("metal", "Metal"), ("mlx", "Apple MLX"), ("vulkan", "Vulkan"),
        ("rocm", "ROCm"), ("sycl", "SYCL"), ("opencl", "OpenCL"), ("npu", "Mobile NPU"),
        ("webgpu", "WebGPU"), ("onnx", "ONNX"), ("cpu", "CPU"),
    ] {
        if blob.contains(kw) { b.push(tag.to_string()); }
    }
    if b.is_empty() { b.push("CPU / Local GPU".into()); }
    b
}

fn infer_platforms(blob: &str, apple: bool, mobile: bool) -> Vec<String> {
    let mut p = Vec::new();
    for (kw, tag) in [
        ("windows", "Windows"), ("linux", "Linux"), ("macos", "macOS"), ("docker", "Docker"),
        ("android", "Android"), ("ios", "iOS"), ("raspberry", "Raspberry Pi"),
    ] {
        if blob.contains(kw) { p.push(tag.to_string()); }
    }
    if p.is_empty() {
        if mobile { p.extend(["Android".into(), "iOS".into()]); }
        else {
            p.extend(["Linux".into(), "Windows".into()]);
            if apple { p.push("macOS".into()); }
        }
    }
    p
}

fn norm_key(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect()
}

fn match_speed_stat<'a>(name: &str, slug: &str, stats: &'a HashMap<String, (f32, usize, String)>) -> Option<&'a (f32, usize, String)> {
    let n_k = norm_key(name);
    let tail_k = norm_key(slug.split('/').last().unwrap_or(slug));
    stats.iter().find(|(bk, _)| {
        let bk_k = norm_key(bk);
        n_k == bk_k || tail_k == bk_k || (bk_k == "mlx" && (tail_k == "mlxlm" || n_k.starts_with("mlx")))
    }).map(|(_, v)| v)
}

fn derive_install_hint(slug: &str, lang: &str) -> String {
    let tail = slug.split('/').last().unwrap_or(slug);
    if lang.eq_ignore_ascii_case("python") {
        format!("pip install {} # https://github.com/{slug}", tail.to_lowercase())
    } else {
        format!("git clone https://github.com/{slug}.git")
    }
}
