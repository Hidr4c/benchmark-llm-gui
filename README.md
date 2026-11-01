# 🚀 LLM Benchmark Explorer

High-performance, modular desktop application written in **Rust** (`eframe`/`egui`) designed to explore, benchmark, filter, and compare **Local and Cloud Large Language Models (LLMs)** across diverse hardware tiers—from resource-constrained embedded edge devices (Micro-IoT, Raspberry Pi, Jetson) up to domestic workstations and flagship GPUs (RTX 4060, RTX 4090, RTX 5090 32 GB).

[![Rust Edition](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/Architecture-Clean%20%26%20IoC-blue.svg)](#architecture)
[![Database](https://img.shields.io/badge/Storage-SQLite%20Bundled%20Embedded-brightgreen.svg)](#storage--zero-daemons)
[![File Limit](https://img.shields.io/badge/File%20Size-%E2%89%A4%20300%20Lines-success.svg)](#engineering-standards)
[![License](https://img.shields.io/badge/License-MIT-lightgrey.svg)](LICENSE)

---

## 📸 Key Capabilities

### 1. Universal Hardware Profiling (Edge to Flagship GPUs)
* **Embedded & Edge Systems**:
  * **Micro-IoT (< 1 GB RAM)**: ESP32-P4, Micro-ARM cores with sub-500 MB quantized models.
  * **Raspberry Pi 4 / 3 (2-4 GB RAM)**: Ultra-compact models running purely on CPU via `llama.cpp`.
  * **Raspberry Pi 5 / Orange Pi 5 (RK3588, 4-8 GB)**: Efficient models leveraging 6 TOPS NPU / CPU.
  * **Jetson Orin Nano (4-8 GB Unified)**: CUDA-accelerated edge inference.
  * **Mini-PCs (Intel N100, 8-16 GB RAM)**: Entry-level local automation hosts.
* **Domestic & High-End PC GPUs**:
  * **RTX 4060 (8 GB VRAM)**, **RTX 4070 (12 GB VRAM)**, **RTX 4080 (16 GB VRAM)**.
  * **RTX 3090 / RTX 4090 (24 GB VRAM)**.
  * **RTX 5090 (32 GB VRAM)**: Full support for 32B/35B high-parameter models at peak memory bandwidth.
* **Embedded Efficiency Metric**:
  * Automatic calculation of the **Efficiency Ratio** ($\text{Intelligence Score} / \text{Footprint in GB}$), highlighting lightweight champion models that punch well above their weight (e.g., SmolLM2 1.7B, Qwen 2.5 0.5B/3B).

### 2. Multi-Source Cross-Scoring & Specialized Benchmarks
* Aggregates multiple authoritative sources to provide balanced, trustworthy intelligence ratings:
  * **Open LLM Leaderboard (v2 & v3 Datasets)**: MMLU-Pro, IFEval, BBH, MATH-5, GPQA.
  * **SWE-Bench Verified**: Software engineering and real-world coding problem resolution.
  * **Berkeley Function Calling & Agentic Benchmarks**: Autonomous tool-use and JSON orchestration.
  * **LLM-Speed Datasets**: Hardware-measured token generation throughput (tokens/sec).
* **Flexible Scoring**: Switch seamlessly between a balanced multi-source average or targeted evaluations (Coding, Reasoning, Tool Calling, Speed).

### 3. Dynamic Live Cloud Discovery & Pricing
* **Frontier Models in Real Time**: Live discovery through the OpenRouter API (`https://openrouter.ai/api/v1/models`) pulling latest 2025/2026 releases (Gemini 4 Argon, GPT-6.1 Terra, Claude 4, DeepSeek V3/R1).
* **Transparent API Costs**: Real-time pricing integration via LiteLLM ($/1M prompt and completion tokens).
* **Direct Comparison**: Benchmark local zero-cost models side-by-side with commercial cloud APIs.

### 4. Model Types & Modalities (Apples-to-Apples Comparisons)
* Categorizes models by their functional modality to avoid comparing incomparable architectures:
  * **📝 Text & Reasoning (LLM)**: Pure language, reasoning, and coding models.
  * **👁️ Multimodal (Vision+Text)**: Vision-language models (Gemini 2.0 Flash, GPT-4o, Pixtral, Qwen 2.5 VL).
  * **🎨 Image Generation**: Diffusion & flow matching models (FLUX.1, SDXL, Midjourney).
  * **🎬 Video Generation**: High-fidelity video generators (Kling Video, CogVideoX, Sora).
  * **🎙️ Audio & Transcription**: Speech-to-text and audio processing (Whisper Large v3, Moonshine).
  * **🔢 Embeddings**: Text and multimodal semantic vectors.
* Color-coded badges and dedicated dropdown filters ensure clean domain-specific evaluations.

### 5. Root Family Grouping (Local & Cloud Anti-Redundancy)
* Eliminates clutter by grouping variants and quantizations (Q4_K_M, Q8_0, FP16) as well as cloud sub-tiers (`GPT 6.1 Sol` & `GPT 6.1 Sol Pro`, `Gemini 2.0 Flash` & `Lite`) under a single canonical parent entry (e.g., `📁 Qwen 2.5 Coder 32B [6 variants]`, `📁 GPT 6.1 Sol [2 variants]`).
* One-click expansion (`▶ / ▼`) to inspect specific quantization levels, file sizes, pricing, and hardware requirements.

### 6. Interactive Graph & Pareto Frontier Analysis (`egui_plot`)
* 2D scatter visualization with automatic computation and rendering of the **Pareto Optimal Frontier**.
* 1-Click Presets:
  * ⭐ *Pareto Frontier (Intelligence vs VRAM)*
  * 🚀 *Throughput vs Intelligence (Tokens/s vs Score)*
  * ⚡ *Embedded Efficiency (Ratio vs Footprint)*
  * 💻 *Coding Performance vs VRAM*
  * 🤖 *Agentic Capabilities vs VRAM*
* **Ergonomic & Bounded Navigation**:
  * Lightweight 2-line hover preview with model modality badges.
  * Click-to-select node with golden halo highlight and docked model inspection drawer.
  * **Bounded Zoom & Panning**: Strict viewport clamping prevents losing orientation in empty space.
  * **🎯 Recentrer View Button**: 1-click view auto-bounds reset (also accessible via canvas double-click).

### 7. Historical Snapshots & Time-Travel
* Every network synchronization creates an immutable timestamped snapshot in the embedded SQLite database.
* Switch snapshot via dropdown to analyze leaderboard shifts and model evolution over time.

### 8. Storage & Zero External Daemons
* Completely self-contained: Local `benchmark.db` powered by `rusqlite` with bundled SQLite compilation.
* Zero external background services, containers, or daemon processes required.

---

## 🛠️ Architecture

The project strictly adheres to **Clean Architecture** and **Inversion of Control (IoC)**. High-level business logic is decoupled from external APIs, database drivers, and presentation frameworks.

```text
src/
├── domain/                    # Enterprise business logic, entities & traits (Ports)
│   ├── hardware.rs            # Target hardware profiles (IoT, Pi, Jetson, discrete GPUs)
│   ├── modality.rs            # Functional modalities (Text, Multimodal, Image, Video, Audio)
│   ├── model.rs               # Domain entities (ModelBenchmark, BenchmarkScores, Quantization)
│   ├── sources.rs             # Registry of 15+ benchmark datasets & leaderboards
│   └── traits.rs              # Interfaces: IBenchmarkRepository, IModelFetcher, IHardwareScorer
├── infrastructure/            # Concrete adapters (Persistence & Network)
│   ├── fetcher/
│   │   ├── cloud_adapter.rs   # OpenRouter & LiteLLM live pricing/catalog fetcher
│   │   ├── hf_adapter.rs      # Hugging Face GGUF model registry
│   │   ├── leaderboard_adapter.rs # Open LLM Leaderboard 2 & 3 dataset parser
│   │   ├── speed_adapter.rs   # LLM-Speed real hardware throughput benchmarks
│   │   ├── vram_benchmark_adapter.rs # Coding & agentic VRAM benchmark adapter
│   │   ├── helpers.rs         # Model tag parsing and scoring helpers
│   │   └── unifier.rs         # Concurrent multi-source fetch orchestrator
│   └── repository/
│       └── sqlite_repo.rs     # Embedded SQLite persistence & snapshot versioning
├── presentation/              # Presentation layer (egui / eframe)
│   ├── app.rs                 # Main GUI state controller and tab navigation
│   ├── filtering.rs           # Filter state machine, search, and family grouping engine
│   ├── theme.rs               # Centralized semantic theme design tokens
│   ├── i18n/                  # Dynamic bilingual system (FR/EN runtime switcher)
│   │   ├── locale.rs          # Language enum & Translations contract
│   │   ├── en.rs              # English dictionary
│   │   └── fr.rs              # French dictionary
│   └── components/
│       ├── detail_view.rs     # Model inspection drawer with Ollama copy snippet
│       ├── filter_bar.rs      # Dropdown filter cockpit and contextual advice
│       ├── graph_types.rs     # Coordinate mapping and Pareto frontier calculator
│       ├── graph_view.rs      # Interactive Pareto plot with dynamic font scaling
│       ├── sources_view.rs    # Benchmark reference library and catalog explorer
│       ├── table_items.rs     # Root family display items builder
│       └── table_view.rs      # Hierarchical data table with family foldout
├── lib.rs
└── main.rs                    # Dual-mode entry point (GUI desktop & Headless CLI)
```

---

## 📏 Engineering Standards

* **Strict File Limit**: Every source file in this codebase is strictly capped at **$\le 300$ lines of code** to guarantee Single Responsibility (SRP) and readability.
* **Zero Hardcoded Catalogs**: No static model catalogs in production code. All entries originate dynamically from upstream APIs or local SQLite snapshots.
* **Testing Discipline**: In-memory SQLite repository tests, edge hardware compatibility validation, and multi-source score blending tests (`cargo test`).

---

## 🚀 Quick Start

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (version 1.80+ recommended)

### Run Desktop GUI
```powershell
cargo run --release
```

### Headless CLI Synchronization
To populate or update the embedded database without launching the GUI:
```powershell
# Synchronizes live datasets, records a new snapshot, and prints Top 5 edge/local models
cargo run --release -- --sync
```

### Run Automated Tests
```powershell
cargo test
```

---

## 📚 Benchmark Sources & References

| Source Name | Focus Area | Access Mechanism |
| :--- | :--- | :--- |
| **Open LLM Leaderboard (v2/v3)** | Academic Reasoning (MMLU-Pro, IFEval, BBH, MATH-5) | Hugging Face Datasets API |
| **SWE-Bench Verified** | Software Engineering & Bug Fixing | Official Leaderboard Dataset |
| **BFCL (Berkeley Function Calling)** | Tool Use, Function Calling, API Control | Benchmark Framework Dataset |
| **LLM-Speed Benchmarks** | Real Hardware Throughput (Tokens/s) | Signed Datasets (RTX 4090, RTX 5090) |
| **Hugging Face Hub (GGUF)** | Quantized Local Models (Q4_K_M, Q8_0) | Official GGUF Hub API |
| **OpenRouter API** | Frontier Models & Real-time Live Discovery | Open REST Endpoint |
| **LiteLLM Pricing** | Cloud API Pricing ($/1M tokens) | Automated GitHub Registry |

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
