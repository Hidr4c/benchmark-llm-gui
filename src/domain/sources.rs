use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceCategory {
    AcademicGeneral,   // Open LLM Leaderboard, MMLU-PRO, LiveBench, ARC-AGI
    CodingDevelopment, // SWE-Bench, LiveCodeBench, HumanEval+
    AgenticTools,      // Agentic Score (RTX 5090), BFCL, Local LLM Benchmark
    RealWorldEconomic, // RLI, SWE-Lancer, SWE-Clar, OSWorld, WebArena, MLE-bench
    HardwareSpeed,     // LLM-Speed, Hardware Benchmarks
    CommunityLibrary,  // Hugging Face Hub (GGUF), Ollama Library
    CloudProprietary,  // LiteLLM Pricing, LMSYS Arena JSON, Artificial Analysis
}

impl SourceCategory {
    pub fn label(&self) -> &'static str {
        match self {
            SourceCategory::AcademicGeneral => "General Intelligence, Reasoning & Uncontaminated Benchmarks",
            SourceCategory::CodingDevelopment => "Code, Software Engineering & Real Bug Fixing",
            SourceCategory::AgenticTools => "Autonomous Agents & Tool-Calling on Dedicated GPUs",
            SourceCategory::RealWorldEconomic => "Real-World Value, Freelance Work & Economic Tasks",
            SourceCategory::HardwareSpeed => "Hardware Throughput, Latency (TTFT) & Speed (tok/s)",
            SourceCategory::CommunityLibrary => "Model Registries & Quantized GGUF Catalogs",
            SourceCategory::CloudProprietary => "Proprietary Cloud Models, Pricing & Human ELO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSourceInfo {
    pub name: &'static str,
    pub category: SourceCategory,
    pub url: &'static str,
    pub description: &'static str,
    pub metrics_measured: &'static str,
    pub is_connected_live: bool,
}

impl BenchmarkSourceInfo {
    pub fn all_sources() -> Vec<BenchmarkSourceInfo> {
        vec![
            // 1. General & Academic
            BenchmarkSourceInfo {
                name: "Open LLM Leaderboard (v2 & v3 Datasets API)",
                category: SourceCategory::AcademicGeneral,
                url: "https://huggingface.co/api/datasets/open-llm-leaderboard/contents/leaderboard",
                description: "World reference benchmark hosted by Hugging Face, queried via datasets-server API.",
                metrics_measured: "MMLU-PRO (Advanced Knowledge), IFEval (Instructions), BBH (Reasoning), MATH Lvl 5, GPQA.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "LiveBench (Contamination-Resistant)",
                category: SourceCategory::AcademicGeneral,
                url: "https://livebench.ai/",
                description: "Monthly benchmark featuring post-cutoff questions to mitigate memorization and contamination.",
                metrics_measured: "Reasoning, mathematics, coding, and comprehension without pre-training data leakage.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "ARC-AGI-2 (Abstract Reasoning)",
                category: SourceCategory::AcademicGeneral,
                url: "https://arcprize.org/leaderboard",
                description: "Out-of-distribution reasoning benchmark discriminating frontier models.",
                metrics_measured: "Visual abstraction and novel puzzle solving capabilities.",
                is_connected_live: false,
            },
            BenchmarkSourceInfo {
                name: "BenchLM.ai (Open-Source Leaderboard)",
                category: SourceCategory::AcademicGeneral,
                url: "https://benchlm.ai/",
                description: "Independent frequently updated leaderboard ranking modern open-weight LLMs.",
                metrics_measured: "Synthetic general score, multi-task ranking.",
                is_connected_live: true,
            },

            // 2. Coding & Software Engineering
            BenchmarkSourceInfo {
                name: "SWE-Bench Verified (Hugging Face API)",
                category: SourceCategory::CodingDevelopment,
                url: "https://huggingface.co/api/datasets/SWE-bench/SWE-bench_Verified/leaderboard",
                description: "Industry gold standard measuring real GitHub issue and pull request resolution.",
                metrics_measured: "Verified patch resolution rate against full project unit test suites.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "LiveCodeBench (Uncontaminated)",
                category: SourceCategory::CodingDevelopment,
                url: "https://livecodebench.github.io/leaderboard.html",
                description: "Competitive coding problems continuously extracted from recent contests.",
                metrics_measured: "Code generation rate without data contamination risk.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "LLM-Bench Scenario Coding",
                category: SourceCategory::CodingDevelopment,
                url: "https://huggingface.co/datasets/llmbenchio/benchmarks-by-vram",
                description: "Empirical code model evaluation grouped by GPU VRAM requirement bracket.",
                metrics_measured: "Coding quality relative to hardware memory footprint.",
                is_connected_live: true,
            },

            // 3. Agents & Tool Calling
            BenchmarkSourceInfo {
                name: "Agentic Score Leaderboard (RTX 5090)",
                category: SourceCategory::AgenticTools,
                url: "https://huggingface.co/datasets/witcheer/agentic-score-leaderboard",
                description: "Dedicated leaderboard evaluating tool-calling capabilities on RTX 5090 32GB GPUs.",
                metrics_measured: "Agentic score, tool execution success rate, function calling efficiency.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "Local LLM Benchmark - ABS (Real Agent Tasks)",
                category: SourceCategory::AgenticTools,
                url: "https://github.com/JConradoN/local-llm-benchmark",
                description: "Evaluation of 19 local LLMs on comprehensive agentic tasks (tools, code, bash, browser).",
                metrics_measured: "Global agentic score, token throughput, and real VRAM consumption.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "Berkeley Function Calling Leaderboard (BFCL)",
                category: SourceCategory::AgenticTools,
                url: "https://gorilla.cs.berkeley.edu/leaderboard.html",
                description: "UC Berkeley standard for assessing external tool invocation and API orchestration.",
                metrics_measured: "Simple, parallel, multiple function calls and irrelevant argument detection.",
                is_connected_live: true,
            },

            // 4. Real-World Economic & Concrete Tasks
            BenchmarkSourceInfo {
                name: "Remote Labor Index (RLI) - Scale AI / CAIS",
                category: SourceCategory::RealWorldEconomic,
                url: "https://labs.scale.com/leaderboard/rli",
                description: "Evaluates AI agent capability to complete genuine freelance projects from Upwork and Fiverr.",
                metrics_measured: "Automation Rate (% of projects completed to client acceptance standard).",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "SWE-Lancer - OpenAI",
                category: SourceCategory::RealWorldEconomic,
                url: "https://github.com/openai/SWELancer-Benchmark",
                description: "Over 1,400 real software dev tasks from Upwork representing $1,000,000 in actual paid bounties.",
                metrics_measured: "Overall Performance Score (%) and Real Dollars Earned ($).",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "SWE-Clar - Upwork",
                category: SourceCategory::RealWorldEconomic,
                url: "https://github.com/upwork/SWE-Clar",
                description: "100 high-value greenfield dev missions (min. $4,000 each) evaluating client clarification & code delivery.",
                metrics_measured: "Implementation Score (%) and Requirement Clarification Score (%).",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "OSWorld - Computer Use Agents",
                category: SourceCategory::RealWorldEconomic,
                url: "https://os-world.github.io/",
                description: "Open-ended computer task evaluation across full operating systems (Ubuntu, Windows, macOS).",
                metrics_measured: "Task Success Rate (%) across desktop applications and system workflows.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "WebArena - Autonomous Web Agents",
                category: SourceCategory::RealWorldEconomic,
                url: "https://github.com/web-arena-x/webarena",
                description: "End-to-end self-hostable web environment assessing multi-page navigation and transactions.",
                metrics_measured: "End-to-end Task Completion Rate (%) from official Google Sheet leaderboard.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "MLE-bench - OpenAI Kaggle Engineering",
                category: SourceCategory::RealWorldEconomic,
                url: "https://github.com/openai/mle-bench",
                description: "Machine learning engineering benchmark based on 75 competitive Kaggle competitions.",
                metrics_measured: "Medal Achievement Rates (Low, Med, High) and Overall All Score (%).",
                is_connected_live: true,
            },

            // 4. Hardware Speed & Real Benchmarks
            BenchmarkSourceInfo {
                name: "LLM-Speed Benchmarks (Signed Datasets)",
                category: SourceCategory::HardwareSpeed,
                url: "https://huggingface.co/datasets/llmspeed/llm-speed-benchmarks",
                description: "Signed real hardware measurements on domestic GPUs and Apple Silicon.",
                metrics_measured: "Decode throughput (tok/s), Time to First Token (TTFT) on RTX 5090, 4090, M4 Max, CPU.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "Data.gouv.fr - Local AI Benchmarks (2,596 runs)",
                category: SourceCategory::HardwareSpeed,
                url: "https://data.gouv.fr/fr/datasets/benchmarks-ia-locale",
                description: "Public hardware inference benchmark dataset covering multiple consumer configurations.",
                metrics_measured: "Throughput (tok/s), TTFT latency, peak VRAM on consumer GPUs and edge boards.",
                is_connected_live: true,
            },

            // 5. GGUF Models & Community Hub
            BenchmarkSourceInfo {
                name: "Hugging Face Hub (GGUF API)",
                category: SourceCategory::CommunityLibrary,
                url: "https://huggingface.co/api/models?filter=gguf",
                description: "Official discovery source for quantized GGUF models.",
                metrics_measured: "Downloads, likes, architectures, tags, and quantization matrices (Q4_K_M, Q8_0).",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "Ollama Library",
                category: SourceCategory::CommunityLibrary,
                url: "https://ollama.com/library",
                description: "Registry for single-command CLI deployment on domestic hosts and edge servers.",
                metrics_measured: "Execution tags (`ollama run qwen2.5:32b`, `mistral:7b`).",
                is_connected_live: true,
            },

            // 6. Cloud Models & Pricing
            BenchmarkSourceInfo {
                name: "LiteLLM Pricing Database",
                category: SourceCategory::CloudProprietary,
                url: "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json",
                description: "Global pricing index tracking exact token rates for all major commercial cloud LLMs.",
                metrics_measured: "Prompt and completion pricing ($/1M tokens), maximum context window.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "Artificial Analysis (LLM Quality & Speed Index)",
                category: SourceCategory::CloudProprietary,
                url: "https://artificialanalysis.ai/api/v2/data/llms/models",
                description: "Independent evaluation tracking median speed, latency, and composite intelligence index.",
                metrics_measured: "Quality index, median output tokens/s, time to first token, pricing.",
                is_connected_live: true,
            },
            BenchmarkSourceInfo {
                name: "LMSYS Chatbot Arena (Daily Snapshots JSON & API)",
                category: SourceCategory::CloudProprietary,
                url: "https://api.wulong.dev/arena-ai-leaderboards/v1/leaderboard?name=text",
                description: "Crowdsourced blind human preference tournament via daily JSON snapshots and REST API.",
                metrics_measured: "Human preference Bradley-Terry ELO rating.",
                is_connected_live: true,
            },
        ]
    }
}
