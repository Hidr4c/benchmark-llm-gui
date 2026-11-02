use eframe::egui;
use crate::domain::model::ModelBenchmark;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn infer_ollama_command(model: &ModelBenchmark) -> String {
    let lower_id = model.id.to_lowercase();
    let lower_name = model.name.to_lowercase();

    if lower_id.contains("qwen2.5-coder-32b") || lower_name.contains("qwen 2.5 coder 32b") {
        "ollama run qwen2.5-coder:32b".into()
    } else if lower_id.contains("qwen2.5-coder-7b") || lower_name.contains("qwen 2.5 coder 7b") {
        "ollama run qwen2.5-coder:7b".into()
    } else if lower_id.contains("qwen2.5-7b") || lower_name.contains("qwen 2.5 7b") {
        "ollama run qwen2.5:7b".into()
    } else if lower_id.contains("qwen2.5-14b") || lower_name.contains("qwen 2.5 14b") {
        "ollama run qwen2.5:14b".into()
    } else if lower_id.contains("qwen2.5-32b") || lower_name.contains("qwen 2.5 32b") {
        "ollama run qwen2.5:32b".into()
    } else if lower_id.contains("deepseek-r1-distill-qwen-14b") {
        "ollama run deepseek-r1:14b".into()
    } else if lower_id.contains("deepseek-r1-distill-llama-8b") {
        "ollama run deepseek-r1:8b".into()
    } else if lower_id.contains("deepseek-r1") || lower_name.contains("deepseek r1") {
        "ollama run deepseek-r1".into()
    } else if lower_id.contains("llama-3.3-70b") || lower_name.contains("llama 3.3 70b") {
        "ollama run llama3.3:70b".into()
    } else if lower_id.contains("llama-3.2-3b") || lower_name.contains("llama 3.2 3b") {
        "ollama run llama3.2:3b".into()
    } else if lower_id.contains("llama-3.2-1b") || lower_name.contains("llama 3.2 1b") {
        "ollama run llama3.2:1b".into()
    } else if lower_id.contains("mistral-7b") || lower_name.contains("mistral 7b") {
        "ollama run mistral:7b".into()
    } else if lower_id.contains("gemma-2-9b") || lower_name.contains("gemma 2 9b") {
        "ollama run gemma2:9b".into()
    } else if lower_id.contains("phi-4") || lower_name.contains("phi 4") {
        "ollama run phi4".into()
    } else {
        let tag = model.id.split('/').last().unwrap_or(&model.name)
            .to_lowercase()
            .replace("-instruct", "")
            .replace("_", "-");
        format!("ollama run {}", tag)
    }
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
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(r.title).strong().size(12.0).color(ThemeTokens::CYAN_EDGE));
            });
            ui.add_space(4.0);

            if model.is_cloud {
                let curl_cmd = format!(
                    "curl https://openrouter.ai/api/v1/chat/completions \\\n  -H \"Authorization: Bearer $API_KEY\" \\\n  -d '{{\"model\": \"{}\", \"messages\": [{{\"role\": \"user\", \"content\": \"Hello!\"}}]}}'",
                    model.id
                );
                ui.label(egui::RichText::new(r.label_api_snippet).size(11.0).color(ThemeTokens::TEXT_MUTED));
                ui.label(egui::RichText::new(&curl_cmd).monospace().size(10.5).color(ThemeTokens::TEXT_PRIMARY));
                ui.add_space(2.0);

                let copy_key = egui::Id::new("copy_curl_time");
                let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
                let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

                let btn_text = if is_recent { r.btn_copied } else { r.btn_copy_curl };
                let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::PRIMARY };
                if ui.button(egui::RichText::new(btn_text).color(btn_color).strong()).clicked() {
                    ui.ctx().copy_text(curl_cmd);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
                }
            } else {
                let ollama_cmd = infer_ollama_command(model);
                let copy_key = egui::Id::new("copy_ollama_time");
                let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
                let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&ollama_cmd).monospace().strong().size(12.0).color(egui::Color32::WHITE));
                    let btn_text = if is_recent { r.btn_copied } else { r.btn_copy_ollama };
                    let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::CYAN_EDGE };
                    if ui.button(egui::RichText::new(btn_text).color(btn_color).strong()).clicked() {
                        ui.ctx().copy_text(ollama_cmd.clone());
                        ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
                    }
                });

                ui.add_space(2.0);
                let cli_cmd = format!("llama-cli -hf {} -p \"Bonjour !\"", model.id);
                ui.label(egui::RichText::new(format!("{} {}", r.cli_example, cli_cmd)).size(10.5).color(ThemeTokens::TEXT_MUTED).monospace());
            }
        });
}
