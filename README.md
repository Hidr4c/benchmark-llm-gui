# LLM Benchmark Explorer

High-performance, modular desktop application written in **Rust** (`eframe`/`egui`) designed to explore, benchmark, filter, and compare **Local and Cloud Foundation Models**, **AI Coding Harnesses**, and **Local Inference Engines** across diverse hardware tiers—from resource-constrained embedded edge devices up to domestic workstations and multi-GPU servers.

[![Latest Release](https://img.shields.io/github/v/release/Hidr4c/benchmark-llm-gui?color=brightgreen&label=Release)](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest)
[![CI & Build](https://github.com/Hidr4c/benchmark-llm-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/Hidr4c/benchmark-llm-gui/actions/workflows/ci.yml)
[![Rust Edition](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/Architecture-Clean%20%26%20IoC-blue.svg)](#architecture)
[![Storage](https://img.shields.io/badge/Storage-Embedded%20SQLite%20Bundled-brightgreen.svg)](#storage--zero-external-daemons)
[![File Limit](https://img.shields.io/badge/File%20Size-%E2%89%A4%20300%20Lines-success.svg)](#industrial-engineering-standards)
[![License](https://img.shields.io/badge/License-MIT-lightgrey.svg)](LICENSE)

---

## Standalone Binaries (Zero Setup Required)

Download pre-compiled, self-contained desktop binaries with zero external dependencies (bundled embedded SQLite database, zero external runtime, zero daemon processes):

| Platform | Target Architecture | Package |
| :--- | :--- | :--- |
| **Windows x64 (Direct EXE)** | `x86_64-pc-windows-msvc` | [**`benchmark-llm-gui-windows-x64.exe`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |
| **Windows x64 (ZIP)** | `x86_64-pc-windows-msvc` | [**`benchmark-llm-gui-windows-x64.zip`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |
| **Linux x64** | `x86_64-unknown-linux-gnu` | [**`benchmark-llm-gui-linux-x64.tar.gz`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |
| **macOS ARM64** | `aarch64-apple-darwin` | [**`benchmark-llm-gui-macos-arm64.tar.gz`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |

---

## Key Capabilities

### 1. Live Hardware Profiling, Simulator & Runtime Optimizer
* **Tiered Hardware Profiles**:
  * **Micro-IoT (< 1 GB RAM)**: Ultra-compact embedded boards with quantized models.
  * **Single-Board Computers (2-8 GB RAM)**: Edge boards running lightweight models purely on CPU/NPU.
  * **Compact PCs & Mobile Devices (4-16 GB Unified/RAM)**: On-device smartphone and mini-PC hosts.
  * **Domestic Discrete Accelerators (8-16 GB VRAM)** & **High-End GPUs (24-32 GB VRAM)**.
  * **Dedicated Workstations (64+ GB Memory)**: High-capacity unified/accelerator environments.
* **Live-Reactive Simulator**:
  * Changing hardware target, system RAM, storage limit, or reserved server/Docker memory updates projected tok/s, memory saturation, OOM stability, and compatible models in real time with zero manual apply step.
  * Includes **Top 4 Clickable Cards**, an **Inline Compatible Models Table** (full-row single-click inspection with column filtering/sorting), and a **Runtime Optimizer** computing exact quantization, KV cache (`F16`, `Q8_0`, `Q4_0`), `n_ctx`, GPU layers (`-ngl`), and ready-to-copy CLI commands.
* **Safe Hardware Auto-Detection**: Inspects available RAM and GPU memory in-process via `catch_unwind` to configure matching hardware filters in one click.

### 2. Multi-Pillar & Media-Isolated Calibrated Benchmarks
* **Dedicated Scoring Pillars**:
  * **General Academic Reasoning**: Standardized MMLU-Pro, MATH Lvl-5, GPQA, IFEval, and Open LLM scores.
  * **Software Engineering & Autonomous Agents**: SWE-bench, coding suites, and tool-calling (BFCL) evaluations, plus dedicated **Dev Agentic** preset.
  * **Isolated Creative & Media Benchmarks**: Dedicated **`ImageBench`**, **`VideoBench`**, and **`AudioBench`** columns integrating VBench, LMArena Image/Video, TTS Arena, OpenASR, and ComfyUI indices—strictly isolated from text LLM Open LLM metrics.
* **Dedicated Columns & Confidence Index**: Separate sortable/filterable **Type** (`Modality` + 6 industrial specialties) and **Confidence** (`High`, `Medium`, `Low`) columns alongside the **Score Calibré**.
* **Dynamic Family Discovery**: Zero hardcoded model catalogs or provider lists; families are derived morphologically at runtime from live repositories and model metadata.

### 3. Dynamic Coding Harnesses & Local Inference Ecosystem
* **AI Coding Harnesses (`Harnais IA`)**:
  * Dynamically aggregates runs across live leaderboards (`SWE-bench`, `Terminal-Bench 1.0/2.0`, `Aider Polyglot`, `Cline/OpenCode CSV`, `KiloBench`) and live community Markdown catalogs enriched with real-time GitHub metadata via `ungh.cc`.
  * Includes a **Same-Model Harness VS Comparator** and unified searchable filter combos.
* **Local Runtimes & Software (`Logiciels Locaux`)**:
  * Dynamically discovers local inference engines, desktop GUIs, server runtimes, and mobile runners from live community awesome catalogs, Hugging Face `llm-speed` benchmark CSVs, and GitHub topic discovery.
  * Ranked leaderboard with hardware filter pills (`Desktop GPU`, `Apple Silicon`, `CPU/Edge`, `Off-Grid Smartphone`).

### 4. Interactive Pareto Frontier (`egui_plot`) & VS Comparator
* **Cloud API & Hardware Pareto Analysis**:
  * Computes non-dominated **Pareto Frontier** champions against VRAM, Size, Speed, Efficiency, or Blended USD API Pricing.
  * Excludes unpriced Cloud endpoints cleanly so missing prices never appear as free `$0.00` champions, with a default **Logarithmic Price Scale ($)** spanning `$0.01` to `$50+/1M` tokens and a one-click Speed-axis switch for media APIs.
* **Dual Side-by-Side Comparator (VS Mode)**: Compare any two models across 5 axes with aligned bars, hardware fit verdicts, and dual-model pipeline estimates.

### 5. Unified Column Header Filters & Self-Updater
* **Consistent Header Controls Across Every Table**: Click headers to sort Ascending/Descending (`^`/`v`), right-click for sorting/reset menus, and filter inline across all tables using `~` (Contains), `=` (Exact), `<=`, `>=`, `!~`, `.*` (Regex), and `!.*` operators.
* **Automatic Release Detection & 1-Click Self-Update**: Checks GitHub Releases automatically on startup and refresh, displays `Mettre à jour vers vX.Y.Z` / `Update version to vX.Y.Z` when newer builds are available, downloads/extracts `.zip` or `.tar.gz` archives safely, validates executable headers, and restarts cleanly.

---

## Architecture

The codebase follows **Clean Architecture** and **Inversion of Control (IoC)**:

```text
src/
├── domain/                    # Entities, scoring strategies, rankers & port traits
│   ├── comparator.rs          # Dual-model hardware fit comparison metrics
│   ├── consistency.rs         # Cross-source anomaly & variance checks
│   ├── hardware*.rs           # Platform tiers & hardware detection abstractions
│   ├── harness.rs             # Coding harness profiles & same-model delta logic
│   ├── model*.rs              # ModelBenchmark, BenchmarkScores & dynamic naming
│   ├── modality.rs            # Text, Multimodal, ImageGen, VideoGen, Audio, 3D
│   ├── runtime*.rs            # LocalRuntimeProfile & KV-cache/n_ctx optimizer
│   ├── scoring.rs             # Multi-source calibration & confidence scoring
│   ├── simulator*.rs          # Hardware rig simulator & compatible model ranker
│   ├── sources*.rs            # Benchmark source & ecosystem registry specs
│   ├── specialty.rs           # Specialized domain detection
│   ├── table_*.rs             # Column defs, filter operators & preferences
│   ├── traits.rs              # Ports: IBenchmarkRepository, IModelFetcher, etc.
│   └── updater.rs             # Semver release parsing & IUpdateChecker port
├── infrastructure/            # Concrete persistence, fetchers & OS adapters
│   ├── fetcher/               # HF, OpenRouter, VBench, LMArena, OpenASR, Awesome & ungh adapters
│   ├── hardware/              # Non-intrusive system RAM/GPU detector
│   ├── preferences/           # SQLite table preferences persistence
│   ├── repository/            # Bundled SQLite snapshot & ecosystem repositories
│   └── updater/               # GitHub Releases API & atomic self-updater
├── presentation/              # Desktop GUI layer (eframe / egui)
│   ├── app.rs                 # Main application loop & state lifecycle
│   ├── ecosystem_state.rs     # Async harness/runtime sync & cache coordinator
│   ├── filtering*.rs          # Multi-dimensional filter service & scope state
│   ├── i18n/                  # Full FR / EN localization dictionaries
│   ├── theme.rs               # Centralized semantic theme tokens
│   ├── updater_state.rs       # Background update polling & modal workflow
│   └── components/            # Table views, Pareto plot, Simulator, Harnesses, Runtimes, Tour
├── lib.rs
└── main.rs                    # Desktop GUI & headless --sync CLI entry point
```

---

## Industrial Engineering Standards

* **Strict File Size Cap**: Source files in `src/` are structured cleanly to stay concise (`<= 300` lines).
* **Zero Hardcoded Catalogs**: Models, model families, coding harnesses, and local runtimes are sourced dynamically from live APIs, leaderboards, and local SQLite snapshots.
* **Zero Missing Glyphs**: Strict ASCII/supported-Unicode UI text policy preventing missing box glyphs across custom UI fonts.
* **Automated Verification**: Covered by 17 unit, BDD, integration, and headless AccessKit/egui UI test suites (`cargo test`).

---

## Quick Start

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (Rust 2024 Edition)

### Run Desktop GUI
```powershell
cargo run --release
```

### Headless CLI Synchronization
Synchronize live benchmarks, cloud APIs, coding harnesses, and local runtimes directly into `benchmark.db`:
```powershell
cargo run --release -- --sync
```

### Run Automated Tests
```powershell
cargo test
```

---

## License

This project is licensed under the [MIT License](LICENSE).
