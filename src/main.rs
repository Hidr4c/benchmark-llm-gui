use std::sync::Arc;
use eframe::egui;
use benchmark_llm_gui::domain::traits::{IBenchmarkRepository, IModelFetcher};
use benchmark_llm_gui::infrastructure::fetcher::unifier::LiveModelFetcher;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::{SqliteRepository, DEFAULT_DB_PATH};
use benchmark_llm_gui::presentation::app::BenchmarkApp;
use benchmark_llm_gui::presentation::theme::ThemeTokens;

#[cfg(target_os = "windows")]
unsafe extern "system" {
    fn SetProcessDpiAwarenessContext(value: isize) -> i32;
}

#[cfg(target_os = "windows")]
const DPI_AWARENESS_CONTEXT_SYSTEM_AWARE: isize = -2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;

    let _guard = rt.enter();

    let fetcher = Arc::new(LiveModelFetcher::default());
    let repo_box = Box::new(SqliteRepository::open(DEFAULT_DB_PATH)?);

    // Support --sync or --cli flag for headless execution
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--sync" || a == "--cli") {
        println!("🚀 Running in CLI / Synchronization mode...");
        let models = rt.block_on(fetcher.fetch_and_unify())?;
        println!("✅ Live data retrieved: {} models found!", models.len());

        let mut repo = repo_box;
        let snap_id = repo.save_snapshot(&models, "CLI Synchronization")?;
        println!("💾 Snapshot #{snap_id} saved in '{DEFAULT_DB_PATH}'!");

        println!("\nTop 5 Global Models (Frontier Cloud & Flagships - All Sources):");
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| {
            let sa = a.intelligence().unwrap_or(0.0);
            let sb = b.intelligence().unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, m) in sorted.iter().take(5).enumerate() {
            let tag = if m.is_cloud { "Cloud API" } else { "Local GGUF" };
            println!("  #{}: {} [{}] | Score: {:.1} (Gen: {:.1}, Code: {:.1}, Agent: {:.1}) | Size: {:.1} GB | Speed: {:.0} tok/s",
                i + 1, m.name, tag, m.intelligence().unwrap_or(0.0),
                m.scores.open_llm.unwrap_or(0.0), m.scores.coding.unwrap_or(0.0), m.scores.agentic.unwrap_or(0.0),
                m.size_gb, m.tokens_per_sec.unwrap_or(0.0)
            );
        }

        println!("\nTop 5 Local / Open-Weight Models (Hardware Flagships - 100% Free & Private):");
        let mut local_flagships: Vec<_> = models.iter().filter(|m| !m.is_cloud).cloned().collect();
        local_flagships.sort_by(|a, b| {
            let sa = a.intelligence().unwrap_or(0.0);
            let sb = b.intelligence().unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen_roots = std::collections::HashSet::new();
        let mut rank = 1;
        for m in &local_flagships {
            if seen_roots.insert(m.name.clone()) {
                println!("  #{}: {} | Score: {:.1} (Code: {:.1}, Agent: {:.1}) | Size: {:.1} GB | VRAM: {:.1} GB | Speed: {:.0} tok/s",
                    rank, m.name, m.intelligence().unwrap_or(0.0),
                    m.scores.coding.unwrap_or(0.0), m.scores.agentic.unwrap_or(0.0),
                    m.size_gb, m.vram_rec_gb, m.tokens_per_sec.unwrap_or(0.0)
                );
                rank += 1;
                if rank > 5 { break; }
            }
        }

        if let Some(r1) = local_flagships.iter().find(|m| m.name.to_lowercase().contains("distill") && m.name.to_lowercase().contains("32b")) {
            println!("\n🔍 Calibrated Distilled Baseline: {} | Score: {:.1} (2025 Distill decayed & quantized)",
                r1.name, r1.intelligence().unwrap_or(0.0)
            );
        }

        println!("\nTop 5 Local Models for Lightweight Edge (Efficiency Score / Size):");
        let mut embedded: Vec<_> = models.into_iter().filter(|m| !m.is_cloud).collect();
        embedded.sort_by(|a, b| {
            b.efficiency_ratio().partial_cmp(&a.efficiency_ratio()).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, m) in embedded.iter().take(5).enumerate() {
            println!("  #{}: {} | Efficiency: {:.1} pts/GB | Size: {:.0} MB | Score: {:.1}",
                i + 1, m.name, m.efficiency_ratio(), m.size_mb(), m.intelligence().unwrap_or(0.0)
            );
        }

        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LLM Benchmark Explorer - Edge & Domestic GPU Profiler")
            .with_inner_size([1240.0, 840.0])
            .with_min_inner_size([640.0, 480.0])
            .with_active(true),
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "LLM Benchmark Explorer",
        native_options,
        Box::new(move |cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.window_rounding = 8.0.into();
            visuals.panel_fill = ThemeTokens::BG_APP;
            visuals.override_text_color = Some(ThemeTokens::TEXT_PRIMARY);
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(BenchmarkApp::new(repo_box, fetcher)))
        }),
    ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
