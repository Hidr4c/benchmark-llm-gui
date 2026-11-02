#[derive(Debug, Clone, Copy)]
pub struct ComparatorTranslations {
    pub tab_label: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub load_detected: &'static str,
    pub target_rig_label: &'static str,
    pub slot_model: &'static str,
    pub empty_slot: &'static str,
    pub clear_slot: &'static str,
    pub preset_duo: &'static str,
    pub preset_8g: &'static str,
    pub preset_flagship: &'static str,
    pub metric_intel: &'static str,
    pub metric_tps: &'static str,
    pub metric_vram: &'static str,
    pub metric_power: &'static str,
    pub metric_ratio: &'static str,
    pub pipeline_title: &'static str,
    pub pipeline_desc: &'static str,
    pub pipeline_fit_ok: &'static str,
    pub pipeline_fit_oom: &'static str,
    pub pipeline_gain: &'static str,
    pub fast_router_label: &'static str,
    pub heavy_reasoner_label: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateTranslations {
    pub btn_check: &'static str,
    pub btn_checking: &'static str,
    pub btn_available: &'static str,
    pub btn_up_to_date: &'static str,
    pub modal_title: &'static str,
    pub current_version: &'static str,
    pub new_version: &'static str,
    pub release_notes_title: &'static str,
    pub btn_download: &'static str,
    pub btn_close: &'static str,
    pub no_notes: &'static str,
    pub tooltip_check: &'static str,
    pub err_check: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfidenceTranslations {
    pub ok_label: &'static str,
    pub ok_tooltip: &'static str,
    pub med_label: &'static str,
    pub med_tooltip: &'static str,
    pub low_label: &'static str,
    pub low_tooltip: &'static str,
    pub col_tooltip: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct AdvFilterTranslations {
    pub panel_title: &'static str,
    pub btn_reset: &'static str,
    pub btn_export: &'static str,
    pub tooltip_export: &'static str,
    pub lbl_license: &'static str,
    pub lbl_modality: &'static str,
    pub lbl_family: &'static str,
    pub lbl_max_ram: &'static str,
    pub tooltip_max_ram: &'static str,
    pub suffix_ram: &'static str,
    pub lbl_min_score: &'static str,
    pub tooltip_min_score: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct DockTranslations {
    pub prompt_pick_second: &'static str,
    pub btn_launch: &'static str,
    pub btn_waiting: &'static str,
    pub tooltip_vs: &'static str,
    pub zoom_label: &'static str,
}

pub const FR_UPDATE: UpdateTranslations = UpdateTranslations {
    btn_check: "MàJ",
    btn_checking: "Vérification...",
    btn_available: "Nouvelle Version Disponible !",
    btn_up_to_date: "A jour",
    modal_title: "Mise à Jour de l'Application",
    current_version: "Version Actuelle :",
    new_version: "Nouvelle Version Disponible :",
    release_notes_title: "Notes de Version :",
    btn_download: "Télécharger la Mise à Jour",
    btn_close: "Fermer",
    no_notes: "Aucune note de version fournie.",
    tooltip_check: "Vérifier la disponibilité d'une nouvelle version sur GitHub Releases",
    err_check: "Erreur lors de la vérification de la mise à jour",
};

pub const EN_UPDATE: UpdateTranslations = UpdateTranslations {
    btn_check: "Update",
    btn_checking: "Checking...",
    btn_available: "New Version Available!",
    btn_up_to_date: "Up to date",
    modal_title: "Application Update",
    current_version: "Current Version:",
    new_version: "New Version Available:",
    release_notes_title: "Release Notes:",
    btn_download: "Download Update",
    btn_close: "Close",
    no_notes: "No release notes provided.",
    tooltip_check: "Check GitHub Releases for new application versions",
    err_check: "Error checking for updates",
};

pub const FR_CONFIDENCE: ConfidenceTranslations = ConfidenceTranslations {
    ok_label: "Élevé (Haute Fiabilité)",
    ok_tooltip: "Indice de Confiance : [OK] ÉLEVÉ\n• 3+ benchmarks croisés concordants\n• Faible variance entre les sources (< 16 pts)\n• Résultat stable, robuste et hautement reproductible.",
    med_label: "Modéré (Fiabilité Moyenne)",
    med_tooltip: "Indice de Confiance : [~] MODÉRÉ\n• Sources de benchmarks limitées ou variance modérée\n• Estimation indicative fiable mais à affiner avec de futurs tests.",
    low_label: "Faible (Prudence)",
    low_tooltip: "Indice de Confiance : [!] FAIBLE\n• Données parcellaires ou forte divergence entre les sources (> 25 pts)\n• Résultat à considérer avec précaution.",
    col_tooltip: "Score Calibré (0-100) : Synthèse multi-sources pondérée.\n\nIndices de Confiance :\n • [OK] Vert = Haute fiabilité (3+ sources concordantes)\n • [~] Jaune = Fiabilité modérée (sources limitées)\n • [!] Rouge = Faible fiabilité (forte divergence)",
};

pub const EN_CONFIDENCE: ConfidenceTranslations = ConfidenceTranslations {
    ok_label: "High (High Reliability)",
    ok_tooltip: "Confidence Index: [OK] HIGH\n• 3+ agreeing cross-referenced benchmarks\n• Low variance between sources (< 16 pts)\n• Highly robust and reproducible result.",
    med_label: "Medium (Moderate Reliability)",
    med_tooltip: "Confidence Index: [~] MODERATE\n• Limited benchmark sources or moderate variance\n• Reliable indicative estimation, subject to refinement.",
    low_label: "Low (Caution)",
    low_tooltip: "Confidence Index: [!] LOW\n• Sparse data or significant divergence between sources (> 25 pts)\n• Use with caution.",
    col_tooltip: "Calibrated Score (0-100): Multi-source weighted synthesis.\n\nConfidence Indices:\n • [OK] Green = High reliability (3+ concordant sources)\n • [~] Yellow = Moderate reliability (limited sources)\n • [!] Red = Low reliability (strong divergence)",
};

pub const FR_ADV_FILTER: AdvFilterTranslations = AdvFilterTranslations {
    panel_title: "FILTRES AVANCÉS & AFFINAGE",
    btn_reset: "Réinitialiser les filtres",
    btn_export: "Exporter CSV",
    tooltip_export: "Exporter les modèles filtrés en fichier CSV",
    lbl_license: "Licence :",
    lbl_modality: "Modalité :",
    lbl_family: "Famille :",
    lbl_max_ram: "RAM Max Allouable :",
    tooltip_max_ram: "Mémoire vive / VRAM maximale requise par le modèle pour s'exécuter",
    suffix_ram: " Go (RAM)",
    lbl_min_score: "Score Min :",
    tooltip_min_score: "Score benchmark minimum requis",
};

pub const EN_ADV_FILTER: AdvFilterTranslations = AdvFilterTranslations {
    panel_title: "ADVANCED FILTERS & REFINEMENT",
    btn_reset: "Reset filters",
    btn_export: "Export CSV",
    tooltip_export: "Export filtered models to CSV file",
    lbl_license: "License:",
    lbl_modality: "Modality:",
    lbl_family: "Family:",
    lbl_max_ram: "Max Allocatable RAM:",
    tooltip_max_ram: "Maximum RAM / VRAM required by model to run",
    suffix_ram: " GB (RAM)",
    lbl_min_score: "Min Score:",
    tooltip_min_score: "Minimum required benchmark score",
};

pub const FR_DOCK: DockTranslations = DockTranslations {
    prompt_pick_second: "(Cochez un 2ème modèle pour lancer)",
    btn_launch: "Lancer le Face-à-Face >",
    btn_waiting: "Face-à-Face (1/2)",
    tooltip_vs: "Cliquer pour comparer en Face-à-Face",
    zoom_label: "Taille :",
};

pub const EN_DOCK: DockTranslations = DockTranslations {
    prompt_pick_second: "(Select a 2nd model to launch)",
    btn_launch: "Launch Face-to-Face >",
    btn_waiting: "Face-to-Face (1/2)",
    tooltip_vs: "Click to compare Face-to-Face",
    zoom_label: "Size :",
};
