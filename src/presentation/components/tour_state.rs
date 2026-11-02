use crate::domain::hardware::HardwarePlatform;
use crate::presentation::filtering::{FilterState, ModelScope};

pub const TOUR_TOTAL_STEPS: usize = 9;

pub struct TourState {
    pub is_active: bool,
    pub current_step: usize,
    pub auto_play: bool,
    pub is_paused: bool,
    pub timer: f32,
    pub show_guide_summary: bool,
    pub completed_steps: [bool; TOUR_TOTAL_STEPS],
    pub transition_timer: Option<f32>,
    pub transition_feedback: Option<String>,
    pub request_clear_selection: bool,
    pub prev_scope: Option<ModelScope>,
    pub prev_hardware: Option<HardwarePlatform>,
    pub prev_adv_cols: Option<bool>,
    pub prev_raw_sources: Option<bool>,
    pub prev_show_adv: Option<bool>,
    pub prev_grouping: Option<bool>,
    pub prev_selected_id: Option<String>,
    pub prev_comp_slots: Option<[Option<String>; 2]>,
    pub prev_sim_gpu: Option<crate::domain::simulator::GpuTier>,
}

impl Default for TourState {
    fn default() -> Self {
        Self {
            is_active: false,
            current_step: 0,
            auto_play: false,
            is_paused: false,
            timer: 0.0,
            show_guide_summary: false,
            completed_steps: [false; TOUR_TOTAL_STEPS],
            transition_timer: None,
            transition_feedback: None,
            request_clear_selection: false,
            prev_scope: None,
            prev_hardware: None,
            prev_adv_cols: None,
            prev_raw_sources: None,
            prev_show_adv: None,
            prev_grouping: None,
            prev_selected_id: None,
            prev_comp_slots: None,
            prev_sim_gpu: None,
        }
    }
}

impl TourState {
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = false;
        self.is_paused = false;
        self.timer = 0.0;
        self.show_guide_summary = false;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.request_clear_selection = true;
        self.reset_trackers();
    }

    pub fn start_autoplay(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = true;
        self.is_paused = false;
        self.timer = 0.0;
        self.show_guide_summary = false;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.request_clear_selection = true;
        self.reset_trackers();
    }

    pub fn close(&mut self) {
        self.is_active = false;
        self.auto_play = false;
        self.is_paused = false;
        self.timer = 0.0;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn jump_to(&mut self, step: usize) {
        self.is_active = true;
        self.current_step = step.min(TOUR_TOTAL_STEPS - 1);
        self.auto_play = false;
        self.is_paused = false;
        self.timer = 0.0;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn next(&mut self, total_steps: usize) {
        if self.current_step < TOUR_TOTAL_STEPS {
            self.completed_steps[self.current_step] = true;
        }
        self.is_paused = false;
        self.transition_timer = None;
        self.transition_feedback = None;
        if self.current_step + 1 < total_steps {
            self.current_step += 1;
            self.timer = 0.0;
            self.reset_trackers();
        } else {
            self.close();
        }
    }

    pub fn prev(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.timer = 0.0;
            self.is_paused = false;
            self.transition_timer = None;
            self.transition_feedback = None;
            self.reset_trackers();
        }
    }

    pub fn validate_current_step(&mut self) {
        if self.is_active && self.transition_timer.is_none() {
            if self.current_step < TOUR_TOTAL_STEPS {
                self.completed_steps[self.current_step] = true;
            }
            self.is_paused = false;
            self.transition_timer = Some(2.5);
            self.transition_feedback = Some("Etape validee avec succes !".into());
        }
    }

    pub fn update_transition(&mut self, dt: f32, total_steps: usize) {
        if self.is_paused {
            return;
        }
        if let Some(ref mut remaining) = self.transition_timer {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.transition_timer = None;
                self.transition_feedback = None;
                self.next(total_steps);
            }
        }
    }

    pub fn on_auto_detect_clicked(&mut self) {
        if self.is_active && self.current_step == 1 && self.transition_timer.is_none() {
            self.completed_steps[1] = true;
            self.transition_timer = Some(3.0);
            self.transition_feedback = Some("Detection materielle reussie !".into());
        }
    }

    pub fn on_recenter_clicked(&mut self) {
        if self.is_active && self.current_step == 6 && self.transition_timer.is_none() {
            self.completed_steps[6] = true;
            self.transition_timer = Some(3.0);
            self.transition_feedback = Some("Graphe recentre sur la frontiere de Pareto !".into());
        }
    }

    pub fn check_and_update_comp_progress(&mut self, slots: &[Option<String>; 2]) {
        if self.is_active && self.current_step == 7 && self.transition_timer.is_none() && !self.is_paused {
            if let Some(prev) = &self.prev_comp_slots {
                if prev != slots {
                    self.completed_steps[7] = true;
                    self.transition_timer = Some(4.0);
                    self.transition_feedback = Some("Modeles selectionnes : face-a-face actif !".into());
                }
            }
            self.prev_comp_slots = Some(slots.clone());
        }
    }

    pub fn check_and_update_sim_progress(&mut self, config: &crate::domain::simulator::HardwareRigConfig) {
        if self.is_active && self.current_step == 8 && self.transition_timer.is_none() {
            if let Some(prev) = self.prev_sim_gpu {
                if prev != config.gpu {
                    self.completed_steps[8] = true;
                    self.transition_timer = Some(3.0);
                    self.transition_feedback = Some("Simulation actualisee en temps reel !".into());
                }
            }
            self.prev_sim_gpu = Some(config.gpu);
        }
    }

    pub fn check_and_update_progress(
        &mut self,
        filters: &FilterState,
        selected_model_id: Option<&str>,
        _total_steps: usize,
    ) -> bool {
        if self.transition_timer.is_some() {
            return false;
        }

        let mut completed = false;
        let mut feedback_msg = None;

        match self.current_step {
            0 => {
                if let Some(prev) = self.prev_scope {
                    if prev != filters.scope {
                        completed = true;
                        feedback_msg = Some("Perimetre de modeles mis a jour !");
                    }
                }
            }
            1 | 2 => {
                if let Some(prev) = self.prev_hardware {
                    if prev != filters.hardware {
                        completed = true;
                        feedback_msg = Some("Filtre materiel applique !");
                    }
                }
            }
            3 => {
                let adv_chg = self.prev_adv_cols.map_or(false, |p| p != filters.show_advanced_columns);
                let raw_chg = self.prev_raw_sources.map_or(false, |p| p != filters.show_raw_sources);
                let pnl_chg = self.prev_show_adv.map_or(false, |p| p != filters.show_advanced);
                if adv_chg || raw_chg || pnl_chg {
                    completed = true;
                    feedback_msg = Some("Colonnes et filtres avances configures !");
                }
            }
            4 => {
                if let Some(prev) = self.prev_grouping {
                    if prev != filters.group_by_root {
                        completed = true;
                        feedback_msg = Some("Affichage par famille bascule !");
                    }
                }
            }
            5 => {
                if selected_model_id.is_some() && (self.prev_selected_id.is_none() || self.prev_selected_id.as_deref() != selected_model_id) {
                    completed = true;
                    feedback_msg = Some("Modele selectionne : fiche ouverte !");
                }
            }
            _ => {}
        }

        self.prev_scope = Some(filters.scope);
        self.prev_hardware = Some(filters.hardware);
        self.prev_adv_cols = Some(filters.show_advanced_columns);
        self.prev_raw_sources = Some(filters.show_raw_sources);
        self.prev_show_adv = Some(filters.show_advanced);
        self.prev_grouping = Some(filters.group_by_root);
        self.prev_selected_id = selected_model_id.map(|s| s.to_string());

        if completed {
            self.completed_steps[self.current_step] = true;
            self.transition_timer = Some(3.0);
            self.transition_feedback = feedback_msg.map(|s| s.to_string());
            true
        } else {
            false
        }
    }

    pub fn completed_count(&self) -> usize {
        self.completed_steps.iter().filter(|&&c| c).count()
    }

    fn reset_trackers(&mut self) {
        self.prev_scope = None;
        self.prev_hardware = None;
        self.prev_adv_cols = None;
        self.prev_raw_sources = None;
        self.prev_show_adv = None;
        self.prev_grouping = None;
        self.prev_selected_id = None;
        self.prev_comp_slots = None;
    }
}
