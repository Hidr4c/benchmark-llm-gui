use eframe::egui;
use crate::presentation::theme::ThemeTokens;

pub struct ComboOption {
    pub key: String,
    pub label: String,
    pub search_blob: String,
}

/// Unified single-field Search + Select dropdown with inline Clear (`X`) and empty-query show-all.
pub fn render_searchable_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    query: &mut String,
    selected_key: &mut Option<String>,
    options: &[ComboOption],
    hint: &str,
    width: f32,
    is_fr: bool,
) -> Option<String> {
    let open_id = egui::Id::new(format!("{id_salt}_open"));
    let show_all_id = egui::Id::new(format!("{id_salt}_show_all"));
    let mut is_open = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(open_id)).unwrap_or(false);
    let mut show_all = ui.ctx().memory_mut(|m| m.data.get_temp::<bool>(show_all_id)).unwrap_or(false);
    let mut newly_selected: Option<String> = None;

    let box_resp = ui.horizontal(|ui| {
        let edit_resp = ui.add(
            egui::TextEdit::singleline(query)
                .id_source(format!("{id_salt}_input"))
                .hint_text(hint)
                .desired_width(width),
        );
        if edit_resp.gained_focus() {
            is_open = true;
            show_all = query.trim().is_empty()
                || selected_key.as_ref().is_some_and(|sk| {
                    options.iter().any(|o| &o.key == sk && o.label.eq_ignore_ascii_case(query.trim()))
                });
        }
        if edit_resp.changed() {
            is_open = true;
            show_all = false;
            if query.trim().is_empty() {
                *selected_key = None;
                show_all = true;
            }
        }
        if !query.is_empty() || selected_key.is_some() {
            let clr_tip = if is_fr { "Effacer et tout afficher" } else { "Clear & show all" };
            if ui.small_button("X").on_hover_text(clr_tip).clicked() {
                query.clear();
                *selected_key = None;
                is_open = true;
                show_all = true;
            }
        }
        let arrow = if is_open { "^" } else { "v" };
        if ui.small_button(arrow).clicked() {
            is_open = !is_open;
            if is_open {
                show_all = query.trim().is_empty()
                    || selected_key.as_ref().is_some_and(|sk| {
                        options.iter().any(|o| &o.key == sk && o.label.eq_ignore_ascii_case(query.trim()))
                    });
            }
        }
    }).response;

    if is_open && !options.is_empty() {
        let q = query.trim().to_lowercase();
        let filtered: Vec<&ComboOption> = if q.is_empty() || show_all {
            options.iter().collect()
        } else {
            options.iter().filter(|o| o.search_blob.contains(&q)).collect()
        };

        let popup_pos = egui::pos2(box_resp.rect.left(), box_resp.rect.bottom() + 2.0);
        let popup_w = (width + 60.0).max(320.0).min(ui.ctx().screen_rect().width() - 24.0);
        let area_resp = egui::Area::new(egui::Id::new(format!("{id_salt}_popup")))
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(ThemeTokens::BG_PANEL)
                    .stroke(egui::Stroke::new(1.2, ThemeTokens::CYAN_EDGE))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.set_min_width(popup_w);
                        ui.set_max_width(popup_w + 80.0);
                        let hdr = if is_fr {
                            format!("{} élément(s) (tapez pour filtrer, X pour tout afficher)", filtered.len())
                        } else {
                            format!("{} item(s) (type to filter, X to show all)", filtered.len())
                        };
                        ui.label(egui::RichText::new(hdr).size(10.5).color(ThemeTokens::TEXT_MUTED));
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_salt(format!("{id_salt}_scroll"))
                            .max_height(260.0)
                            .show(ui, |ui| {
                                if filtered.is_empty() {
                                    ui.label(egui::RichText::new(if is_fr { "Aucun résultat." } else { "No match." }).color(ThemeTokens::TEXT_MUTED));
                                } else {
                                    for opt in filtered.iter().take(150) {
                                        let is_sel = selected_key.as_deref() == Some(&opt.key);
                                        if ui.selectable_label(is_sel, &opt.label).clicked() {
                                            *selected_key = Some(opt.key.clone());
                                            *query = opt.label.clone();
                                            newly_selected = Some(opt.key.clone());
                                            is_open = false;
                                            show_all = false;
                                        }
                                    }
                                }
                            });
                    });
            });

        let pointer_down = ui.ctx().input(|i| i.pointer.any_pressed());
        if pointer_down {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                if !box_resp.rect.contains(pos) && !area_resp.response.rect.contains(pos) {
                    is_open = false;
                }
            }
        }
    }

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(open_id, is_open);
        m.data.insert_temp(show_all_id, show_all);
    });
    newly_selected
}
