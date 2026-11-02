#[derive(Debug, Clone, Copy)]
pub struct SyncErrorTranslations {
    pub dialog_title: &'static str,
    pub dialog_desc: &'static str,
    pub trace_label: &'static str,
    pub btn_github_issue: &'static str,
    pub btn_copy_report: &'static str,
    pub copied_notice: &'static str,
    pub btn_close: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ContactTranslations {
    pub btn_contact: &'static str,
    pub dialog_title: &'static str,
    pub dialog_desc: &'static str,
    pub section_github: &'static str,
    pub btn_suggest_feature: &'static str,
    pub btn_report_bug: &'static str,
    pub section_anonymous: &'static str,
    pub anonymous_desc: &'static str,
    pub btn_copy_anonymous_feedback: &'static str,
    pub copied_feedback: &'static str,
    pub btn_close: &'static str,
}

pub const FR_SYNC_ERROR: SyncErrorTranslations = SyncErrorTranslations {
    dialog_title: "[!] Incident lors de l'Actualisation des Données",
    dialog_desc: "Une erreur est survenue pendant la récupération des modèles distants (API Hugging Face, OpenLLM ou LiteLLM). La structure des données a peut-être changé.",
    trace_label: "Détail technique de l'erreur :",
    btn_github_issue: "Signaler sur GitHub (Issue pré-remplie)",
    btn_copy_report: "Copier le rapport complet (Sans compte GitHub)",
    copied_notice: "✓ Rapport d'incident copié dans le presse-papier !",
    btn_close: "Fermer",
};

pub const EN_SYNC_ERROR: SyncErrorTranslations = SyncErrorTranslations {
    dialog_title: "[!] Data Synchronization Incident",
    dialog_desc: "An error occurred while fetching remote models from Hugging Face, OpenLLM, or LiteLLM. Remote schemas or API endpoints might have changed.",
    trace_label: "Technical error trace:",
    btn_github_issue: "Report on GitHub (Prefilled Issue)",
    btn_copy_report: "Copy Error Report (Without GitHub Account)",
    copied_notice: "✓ Error report copied to clipboard!",
    btn_close: "Close",
};

pub const FR_CONTACT: ContactTranslations = ContactTranslations {
    btn_contact: "Contact & Retours",
    dialog_title: "Contact, Idées & Retours Utilisateur",
    dialog_desc: "Vous avez une suggestion de fonctionnalité, un nouveau modèle à intégrer ou une anomalie à signaler ?",
    section_github: "Via GitHub (Suivi officiel en direct) :",
    btn_suggest_feature: "Proposer une Idée / Nouveau Modèle",
    btn_report_bug: "Signaler une Anomalie / Bug",
    section_anonymous: "Sans Compte GitHub (Retour Anonyme) :",
    anonymous_desc: "Pas de compte GitHub ? Aucun problème : copiez un canevas de retour pré-rempli et envoyez-le par mail ou dans nos formulaires.",
    btn_copy_anonymous_feedback: "Copier le canevas de retour",
    copied_feedback: "✓ Canevas de retour copié !",
    btn_close: "Fermer",
};

pub const EN_CONTACT: ContactTranslations = ContactTranslations {
    btn_contact: "Contact & Feedback",
    dialog_title: "Contact, Ideas & User Feedback",
    dialog_desc: "Do you have a feature suggestion, a new model to benchmark, or an anomaly to report?",
    section_github: "Via GitHub (Official issue tracker):",
    btn_suggest_feature: "Propose an Idea / New Model",
    btn_report_bug: "Report a Bug / Issue",
    section_anonymous: "Without GitHub Account (Anonymous Feedback):",
    anonymous_desc: "No GitHub account? No problem: copy a pre-formatted feedback template to share via email or discussion boards.",
    btn_copy_anonymous_feedback: "Copy Feedback Template",
    copied_feedback: "✓ Feedback template copied!",
    btn_close: "Close",
};
