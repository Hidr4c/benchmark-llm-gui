use crate::domain::hardware::HardwarePlatform;
use crate::presentation::filtering::{FilterState, ModelScope};

pub struct TourState {
    pub is_active: bool,
    pub current_step: usize,
    pub auto_play: bool,
    pub timer: f32,
    pub show_guide_summary: bool,
    pub completed_steps: [bool; 7],
    pub transition_timer: Option<f32>,
    pub transition_feedback: Option<String>,
    pub prev_scope: Option<ModelScope>,
    pub prev_hardware: Option<HardwarePlatform>,
    pub prev_grouping: Option<bool>,
    pub prev_selected_id: Option<String>,
    pub prev_sim_gpu: Option<crate::domain::simulator::GpuTier>,
}

impl Default for TourState {
    fn default() -> Self {
        Self {
            is_active: false,
            current_step: 0,
            auto_play: false,
            timer: 0.0,
            show_guide_summary: false,
            completed_steps: [false; 7],
            transition_timer: None,
            transition_feedback: None,
            prev_scope: None,
            prev_hardware: None,
            prev_grouping: None,
            prev_selected_id: None,
            prev_sim_gpu: None,
        }
    }
}

impl TourState {
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = false;
        self.timer = 0.0;
        self.show_guide_summary = false;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn start_autoplay(&mut self) {
        self.is_active = true;
        self.current_step = 0;
        self.auto_play = true;
        self.timer = 0.0;
        self.show_guide_summary = false;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn close(&mut self) {
        self.is_active = false;
        self.auto_play = false;
        self.timer = 0.0;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn jump_to(&mut self, step: usize) {
        self.is_active = true;
        self.current_step = step.min(6);
        self.auto_play = false;
        self.timer = 0.0;
        self.transition_timer = None;
        self.transition_feedback = None;
        self.reset_trackers();
    }

    pub fn next(&mut self, total_steps: usize) {
        self.completed_steps[self.current_step] = true;
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
            self.transition_timer = None;
            self.transition_feedback = None;
            self.reset_trackers();
        }
    }

    pub fn update_transition(&mut self, dt: f32, total_steps: usize) {
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
            self.transition_timer = Some(3.5);
            self.transition_feedback = Some("Détection matérielle réussie !".into());
        }
    }

    pub fn on_recenter_clicked(&mut self) {
        if self.is_active && self.current_step == 5 && self.transition_timer.is_none() {
            self.completed_steps[5] = true;
            self.transition_timer = Some(3.5);
            self.transition_feedback = Some("Graphe recentré sur la frontière de Pareto !".into());
        }
    }

    pub fn check_and_update_sim_progress(&mut self, config: &crate::domain::simulator::HardwareRigConfig) {
        if self.is_active && self.current_step == 6 && self.transition_timer.is_none() {
            if let Some(prev) = self.prev_sim_gpu {
                if prev != config.gpu {
                    self.completed_steps[6] = true;
                    self.transition_timer = Some(3.5);
                    self.transition_feedback = Some("Simulation actualisée en temps réel !".into());
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
                        feedback_msg = Some("Périmètre de modèles mis à jour !");
                    }
                }
            }
            1 => {
                if let Some(prev) = self.prev_hardware {
                    if prev != filters.hardware {
                        completed = true;
                        feedback_msg = Some("Matériel actualisé !");
                    }
                }
            }
            2 => {
                if let Some(prev) = self.prev_hardware {
                    if prev != filters.hardware {
                        completed = true;
                        feedback_msg = Some("Filtre matériel appliqué !");
                    }
                }
            }
            3 => {
                if let Some(prev) = self.prev_grouping {
                    if prev != filters.group_by_root {
                        completed = true;
                        feedback_msg = Some("Affichage par famille basculé !");
                    }
                }
            }
            4 => {
                if selected_model_id.is_some() && self.prev_selected_id.as_deref() != selected_model_id {
                    completed = true;
                    feedback_msg = Some("Modèle sélectionné : fiche ouverte !");
                }
            }
            _ => {}
        }

        self.prev_scope = Some(filters.scope);
        self.prev_hardware = Some(filters.hardware);
        self.prev_grouping = Some(filters.group_by_root);
        self.prev_selected_id = selected_model_id.map(|s| s.to_string());

        if completed {
            self.completed_steps[self.current_step] = true;
            self.transition_timer = Some(3.5);
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
        self.prev_grouping = None;
        self.prev_selected_id = None;
    }
}
