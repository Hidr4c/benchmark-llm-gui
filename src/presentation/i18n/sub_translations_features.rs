#[derive(Debug, Clone, Copy)]
pub struct ColumnModeTranslations {
    pub col_mode_simple: &'static str,
    pub col_mode_detailed: &'static str,
    pub col_mode_strict: &'static str,
    pub col_mode_raw: &'static str,
    pub col_mode_tooltip: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct RunnerTranslations {
    pub title: &'static str,
    pub btn_copy_ollama: &'static str,
    pub btn_copied: &'static str,
    pub label_api_snippet: &'static str,
    pub btn_copy_curl: &'static str,
    pub cli_example: &'static str,
    pub tab_ollama: &'static str,
    pub tab_lm_studio: &'static str,
    pub tab_llama_cpp: &'static str,
    pub tab_vllm: &'static str,
    pub btn_copy_lms: &'static str,
    pub btn_open_lms: &'static str,
    pub btn_copy_llama: &'static str,
    pub btn_copy_vllm: &'static str,
    pub btn_open_hf: &'static str,
    pub hint_ollama: &'static str,
    pub hint_lms: &'static str,
    pub hint_llama_cpp: &'static str,
    pub hint_vllm: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct RoiTranslations {
    pub title: &'static str,
    pub volume_label: &'static str,
    pub volume_suffix: &'static str,
    pub cost_local: &'static str,
    pub cost_cloud: &'static str,
    pub roi_gain: &'static str,
    pub roi_cheaper_cloud: &'static str,
    pub assumptions: &'static str,
}

pub const FR_COL_MODES: ColumnModeTranslations = ColumnModeTranslations {
    col_mode_simple: "Évaluation Générale (Simple)",
    col_mode_detailed: "Évaluation Complète (Avancée)",
    col_mode_strict: "Strictement Vérifié (Haute Fiabilité)",
    col_mode_raw: "Sources Brutes (Benchmarks)",
    col_mode_tooltip: "Choisir le niveau de détail du tableau comparatif",
};

pub const EN_COL_MODES: ColumnModeTranslations = ColumnModeTranslations {
    col_mode_simple: "General Evaluation (Simple)",
    col_mode_detailed: "Detailed Evaluation (Advanced)",
    col_mode_strict: "Strict Verified (High Reliability)",
    col_mode_raw: "Raw Benchmark Sources",
    col_mode_tooltip: "Select comparison table level of detail",
};

pub const FR_RUNNER: RunnerTranslations = RunnerTranslations {
    title: "Exécution Immédiate (CLI & Local)",
    btn_copy_ollama: "Copier Ollama",
    btn_copied: "✓ Copié !",
    label_api_snippet: "Exemple d'appel API :",
    btn_copy_curl: "Copier cURL",
    cli_example: "Commande :",
    tab_ollama: "Ollama",
    tab_lm_studio: "LM Studio",
    tab_llama_cpp: "llama.cpp",
    tab_vllm: "vLLM",
    btn_copy_lms: "Copier commande lms",
    btn_open_lms: "Ouvrir dans LM Studio",
    btn_copy_llama: "Copier llama-cli",
    btn_copy_vllm: "Copier commande vLLM",
    btn_open_hf: "Page Hugging Face",
    hint_ollama: "Serveur en arrière-plan, API REST sur localhost:11434",
    hint_lms: "Application de bureau avec chat, serveur OpenAI sur localhost:1234",
    hint_llama_cpp: "Moteur C/C++ ultra-rapide sans dépendance, zéro overhead VRAM",
    hint_vllm: "Serveur GPU haute performance avec PagedAttention pour production",
};

pub const EN_RUNNER: RunnerTranslations = RunnerTranslations {
    title: "Direct Execution (CLI & Local)",
    btn_copy_ollama: "Copy Ollama",
    btn_copied: "✓ Copied!",
    label_api_snippet: "API Request Snippet:",
    btn_copy_curl: "Copy cURL",
    cli_example: "Command:",
    tab_ollama: "Ollama",
    tab_lm_studio: "LM Studio",
    tab_llama_cpp: "llama.cpp",
    tab_vllm: "vLLM",
    btn_copy_lms: "Copy lms Command",
    btn_open_lms: "Open in LM Studio",
    btn_copy_llama: "Copy llama-cli",
    btn_copy_vllm: "Copy vLLM Command",
    btn_open_hf: "Hugging Face Page",
    hint_ollama: "Background daemon, REST API on localhost:11434",
    hint_lms: "Desktop GUI with chat, OpenAI-compatible server on localhost:1234",
    hint_llama_cpp: "Ultra-fast C/C++ runtime with zero runtime overhead",
    hint_vllm: "High-throughput GPU server with PagedAttention for production",
};

pub const FR_ROI: RoiTranslations = RoiTranslations {
    title: "Arbitrage & Simulation Économique (Local vs Cloud)",
    volume_label: "Volume mensuel simulé :",
    volume_suffix: " req/mois",
    cost_local: "Coût Électricité Local :",
    cost_cloud: "Équivalent API Cloud :",
    roi_gain: "Économie nette estimée :",
    roi_cheaper_cloud: "L'API Cloud reste plus économique à ce volume.",
    assumptions: "(Basé sur 1 050 tokens/req. et 0,25 €/kWh)",
};

pub const EN_ROI: RoiTranslations = RoiTranslations {
    title: "Economic Arbitrage & Simulation (Local vs Cloud)",
    volume_label: "Simulated monthly volume:",
    volume_suffix: " req/month",
    cost_local: "Local Electricity Cost:",
    cost_cloud: "Cloud API Equivalent:",
    roi_gain: "Estimated net savings:",
    roi_cheaper_cloud: "Cloud API remains more economical at this volume.",
    assumptions: "(Based on 1,050 tokens/req. and €0.25/kWh)",
};
