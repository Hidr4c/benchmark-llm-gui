use std::sync::Arc;
use eframe::egui;
use benchmark_llm_gui::infrastructure::fetcher::unifier::LiveModelFetcher;
use benchmark_llm_gui::infrastructure::repository::sqlite_repo::{SqliteRepository, DEFAULT_DB_PATH};
use benchmark_llm_gui::presentation::app::BenchmarkApp;
use benchmark_llm_gui::presentation::components::top_bar::AppTab;
use benchmark_llm_gui::presentation::i18n::Language;

enum CapturePhase {
    Init(usize),
    PrepareTable(usize),
    CaptureTable,
    WaitTable,
    PrepareGraph(usize),
    CaptureGraph,
    WaitGraph,
    PrepareComp(usize),
    CaptureComp,
    WaitComp,
    PrepareSim(usize),
    CaptureSim,
    WaitSim,
    PrepareSources(usize),
    CaptureSources,
    WaitSources,
    PrepareGuide(usize),
    CaptureGuide,
    WaitGuide,
    Done,
}

struct ScreenCapturer {
    app: BenchmarkApp,
    phase: CapturePhase,
}

impl ScreenCapturer {
    fn new(app: BenchmarkApp) -> Self {
        Self {
            app,
            phase: CapturePhase::Init(0),
        }
    }

    fn save_color_image(name: &str, img: &egui::ColorImage) {
        let w = img.width() as u32;
        let h = img.height() as u32;
        let mut bytes = Vec::with_capacity((w * h * 4) as usize);
        for pixel in &img.pixels {
            bytes.push(pixel.r());
            bytes.push(pixel.g());
            bytes.push(pixel.b());
            bytes.push(pixel.a());
        }
        for dest in &["store-assets/promo-v1.1.0-standard", "store-assets/promo-v1.1.0-short"] {
            let path = format!("{dest}/{name}");
            let _ = image::save_buffer(&path, &bytes, w, h, image::ExtendedColorType::Rgba8);
            println!("  [CAPTURED FRESH] -> {path} ({w}x{h})");
        }
    }
}

impl eframe::App for ScreenCapturer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        <BenchmarkApp as eframe::App>::update(&mut self.app, ctx, frame);

        let mut captured_img = None;
        ctx.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    captured_img = Some(image.clone());
                }
            }
        });

        match &mut self.phase {
            CapturePhase::Init(frames) => {
                *frames += 1;
                if *frames >= 8 {
                    self.phase = CapturePhase::PrepareTable(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareTable(f) => {
                self.app.set_active_tab(AppTab::Table);
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureTable;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureTable => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitTable;
                ctx.request_repaint();
            }
            CapturePhase::WaitTable => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-table.png", &img);
                    self.phase = CapturePhase::PrepareGraph(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareGraph(f) => {
                self.app.set_active_tab(AppTab::Graph);
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureGraph;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureGraph => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitGraph;
                ctx.request_repaint();
            }
            CapturePhase::WaitGraph => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-graph.png", &img);
                    self.phase = CapturePhase::PrepareComp(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareComp(f) => {
                self.app.set_active_tab(AppTab::Comparator);
                let models = self.app.all_models();
                let m1 = models.iter().find(|m| m.name.contains("DeepSeek-R1") || m.id.contains("DeepSeek-R1")).or_else(|| models.first());
                let m2 = models.iter().find(|m| m.name.contains("Qwen3-Coder") || m.id.contains("Qwen3-Coder")).or_else(|| models.get(1));
                self.app.set_comparator_slots(m1.map(|m| m.id.clone()), m2.map(|m| m.id.clone()));
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureComp;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureComp => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitComp;
                ctx.request_repaint();
            }
            CapturePhase::WaitComp => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-comp.png", &img);
                    self.phase = CapturePhase::PrepareSim(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareSim(f) => {
                self.app.set_active_tab(AppTab::Simulator);
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureSim;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureSim => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitSim;
                ctx.request_repaint();
            }
            CapturePhase::WaitSim => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-sim.png", &img);
                    self.phase = CapturePhase::PrepareSources(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareSources(f) => {
                self.app.set_active_tab(AppTab::Sources);
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureSources;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureSources => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitSources;
                ctx.request_repaint();
            }
            CapturePhase::WaitSources => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-sources.png", &img);
                    self.phase = CapturePhase::PrepareGuide(0);
                }
                ctx.request_repaint();
            }
            CapturePhase::PrepareGuide(f) => {
                self.app.set_active_tab(AppTab::Tutorial);
                *f += 1;
                if *f >= 3 {
                    self.phase = CapturePhase::CaptureGuide;
                }
                ctx.request_repaint();
            }
            CapturePhase::CaptureGuide => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.phase = CapturePhase::WaitGuide;
                ctx.request_repaint();
            }
            CapturePhase::WaitGuide => {
                if let Some(img) = captured_img {
                    Self::save_color_image("app-guide.png", &img);
                    self.phase = CapturePhase::Done;
                }
                ctx.request_repaint();
            }
            CapturePhase::Done => {
                println!("All 6 app screens captured and updated successfully!");
                std::process::exit(0);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
    let _guard = rt.enter();

    let fetcher = Arc::new(LiveModelFetcher::default());
    let repo_box = Box::new(SqliteRepository::open(DEFAULT_DB_PATH)?);
    let mut app = BenchmarkApp::new(repo_box, fetcher);
    app.language = Language::English;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Screen Capturer")
            .with_inner_size([1240.0, 840.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Screen Capturer",
        native_options,
        Box::new(|_cc| Ok(Box::new(ScreenCapturer::new(app)))),
    ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
