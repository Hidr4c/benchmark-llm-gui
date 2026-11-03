use eframe::egui;
use crate::domain::model::ModelBenchmark;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn infer_ollama_command(model: &ModelBenchmark) -> String {
    if let Some(tag) = &model.ollama_tag {
        return format!("ollama run {tag}");
    }
    if model.id.contains('/') {
        return format!("ollama run hf.co/{}", model.id);
    }
    format!("ollama run {}", model.id)
}

pub fn render_runner_card(ui: &mut egui::Ui, model: &ModelBenchmark, lang: Language) {
    let r = lang.runner();
    ui.add_space(4.0);
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(16, 22, 34))
        .stroke(egui::Stroke::new(1.0, ThemeTokens::BG_PANEL))
        .inner_margin(8.0)
        .rounding(6.0)
        .show(ui, |ui| {
            if model.is_cloud {
                render_cloud_runner(ui, model, r);
            } else {
                render_local_runner(ui, model, r);
            }
        });
}

fn render_cloud_runner(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    ui.label(egui::RichText::new(r.title).strong().size(12.0).color(ThemeTokens::CYAN_EDGE));
    ui.add_space(4.0);

    let curl_cmd = format!(
        "curl https://openrouter.ai/api/v1/chat/completions \\\n  -H \"Authorization: Bearer $API_KEY\" \\\n  -d '{{\"model\": \"{}\", \"messages\": [{{\"role\": \"user\", \"content\": \"Hello!\"}}]}}'",
        model.id
    );
    ui.label(egui::RichText::new(r.label_api_snippet).size(11.0).color(ThemeTokens::TEXT_MUTED));
    ui.label(egui::RichText::new(&curl_cmd).monospace().size(10.5).color(ThemeTokens::TEXT_PRIMARY));
    ui.add_space(3.0);

    let copy_key = egui::Id::new("copy_curl_time");
    let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
    let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

    let btn_text = if is_recent { r.btn_copied } else { r.btn_copy_curl };
    let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::PRIMARY };
    if ui.button(egui::RichText::new(btn_text).color(btn_color).strong()).clicked() {
        ui.ctx().copy_text(curl_cmd);
        ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
    }
}

fn render_local_runner(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    let runner_tab_key = egui::Id::new("detail_runner_selected_tab");
    let selected_tab = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(runner_tab_key)).unwrap_or(0);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(r.title).strong().size(12.0).color(ThemeTokens::CYAN_EDGE));
        ui.add_space(8.0);

        let tabs = [
            (0, r.tab_ollama),
            (1, r.tab_lm_studio),
            (2, r.tab_llama_cpp),
            (3, r.tab_vllm),
        ];

        for (idx, label) in tabs {
            let is_sel = selected_tab == idx;
            let text = egui::RichText::new(label).size(11.0);
            let resp = if is_sel {
                ui.button(text.strong().color(ThemeTokens::CYAN_EDGE))
            } else {
                ui.button(text.color(ThemeTokens::TEXT_MUTED))
            };
            if resp.clicked() {
                ui.ctx().memory_mut(|m| m.data.insert_temp(runner_tab_key, idx));
            }
        }
    });

    ui.add_space(4.0);

    match selected_tab {
        1 => render_lm_studio_tab(ui, model, r),
        2 => render_llama_cpp_tab(ui, model, r),
        3 => render_vllm_tab(ui, model, r),
        _ => render_ollama_tab(ui, model, r),
    }
}

fn render_ollama_tab(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    let ollama_cmd = infer_ollama_command(model);
    render_copyable_command(ui, &ollama_cmd, r.btn_copy_ollama, r.btn_copied, r.hint_ollama);
}

fn render_lm_studio_tab(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    let lms_cmd = format!("lms get {}", model.id);
    render_copyable_command(ui, &lms_cmd, r.btn_copy_lms, r.btn_copied, r.hint_lms);

    ui.horizontal(|ui| {
        let lms_url = format!("lmstudio://open?model={}", model.id);
        if ui.button(egui::RichText::new(r.btn_open_lms).color(ThemeTokens::GOLD_5090).size(10.5)).on_hover_text(r.hint_lms).clicked() {
            let _ = open::that(&lms_url);
        }
        if !model.hf_url.is_empty() && ui.button(egui::RichText::new(r.btn_open_hf).size(10.5)).clicked() {
            let _ = open::that(&model.hf_url);
        }
    });
}

fn render_llama_cpp_tab(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    let cli_cmd = format!("llama-cli -hf {} -p \"Bonjour !\"", model.id);
    render_copyable_command(ui, &cli_cmd, r.btn_copy_llama, r.btn_copied, r.hint_llama_cpp);
}

fn render_vllm_tab(ui: &mut egui::Ui, model: &ModelBenchmark, r: &'static crate::presentation::i18n::RunnerTranslations) {
    let vllm_cmd = format!("vllm serve {} --max-model-len 8192", model.id);
    render_copyable_command(ui, &vllm_cmd, r.btn_copy_vllm, r.btn_copied, r.hint_vllm);
}

fn render_copyable_command(ui: &mut egui::Ui, cmd: &str, btn_label: &str, copied_label: &str, hint: &str) {
    let copy_key = egui::Id::new(format!("copy_{}", cmd));
    let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
    let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(cmd).monospace().strong().size(11.5).color(egui::Color32::WHITE));
        let btn_text = if is_recent { copied_label } else { btn_label };
        let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::CYAN_EDGE };
        if ui.button(egui::RichText::new(btn_text).color(btn_color).strong().size(11.0)).clicked() {
            ui.ctx().copy_text(cmd.to_string());
            ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
        }
    });
    ui.label(egui::RichText::new(hint).size(10.0).color(ThemeTokens::TEXT_MUTED));
}
