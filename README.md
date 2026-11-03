# LLM Benchmark Explorer

High-performance, modular desktop application written in **Rust** (`eframe`/`egui`) designed to explore, benchmark, filter, and compare **Local and Cloud Foundation Models** across diverse hardware tiers—from resource-constrained embedded edge devices up to domestic workstations and high-capacity accelerator setups.

[![Latest Release](https://img.shields.io/badge/Release-v1.1.0-brightgreen.svg)](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest)
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
| **Windows x64** | `x86_64-pc-windows-msvc` | [**`benchmark-llm-gui-windows-x64.zip`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |
| **Linux x64** | `x86_64-unknown-linux-gnu` | [**`benchmark-llm-gui-linux-x64.tar.gz`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |
| **macOS ARM64** | `aarch64-apple-darwin` | [**`benchmark-llm-gui-macos-arm64.tar.gz`**](https://github.com/Hidr4c/benchmark-llm-gui/releases/latest) |

---

## Key Capabilities

### 1. Universal Hardware Profiling & Simulation
* **Tiered Hardware Profiles**:
  * **Micro-IoT (< 1 GB RAM)**: Ultra-compact embedded boards with quantized models.
  * **Single-Board Computers (2-8 GB RAM)**: Edge boards running lightweight models purely on CPU/NPU.
  * **Compact PCs & Mobile Devices (4-16 GB Unified/RAM)**: On-device mobile and compact automation hosts.
  * **Domestic Discrete Accelerators (8-16 GB Memory)**: Mainstream discrete consumer cards.
  * **High-End Enthusiast Accelerators (24-32 GB Memory)**: Capable of running large parameter local models at high throughput.
  * **Dedicated Multi-GPU Workstations (64+ GB Memory)**: High-capacity multi-accelerator environments.
* **Hardware Rig Simulator**:
  * Simulates physical target configurations (memory capacity, memory bus bandwidth, compute allocation).
  * Predicts real inference throughput (tokens/second), memory saturation percentage, and OOM (Out Of Memory) crash risk.
  * Multi-scenario resource reservation: simulates dedicated AI workstations vs. multi-service servers/NAS (reserving system RAM for OS, containers, and web servers).
  * Storage footprint constraints: filters models by on-disk storage size to suit embedded and mobile constraints.
* **Safe Hardware Auto-Detection**:
  * Built-in, non-intrusive hardware detector inspecting available system RAM and accelerator memory in a protected sandbox (`catch_unwind`), automatically pre-configuring matching hardware filters in one click.

### 2. Multi-Pillar Calibrated Benchmarks & Trust Metrics
* **Balanced Cross-Scoring**: Blends scientific evaluations across multiple independent pillars:
  * **General Academic Reasoning**: Standardized multiple-choice and multi-turn reasoning suites.
  * **Software Engineering & Coding**: Real-world bug fixing and repository-level issue resolution.
  * **Autonomous Agents & Tool Calling**: Structured function calling, API invocation, and multi-step tool orchestration.
  * **Real-World Economic Viability**: Cost per task and operational efficiency.
* **Mathematical Calibration (Base 0-100)**: Normalizes raw scientific test scores into an intuitive, consistent 0-100 scale.
* **Variance-Based Confidence Index**:
  * `[OK] High Reliability`: Cross-referenced across 3+ independent benchmark sources with low variance.
  * `[~] Medium Reliability`: Validated by at least 2 concordant sources.
  * `[!] Caution / Preliminary`: Single-source score or high score divergence.

### 3. Specialized Industrial Domains (Direct Toolbar Filtering)
* Foundation models excel in distinct vertical capabilities. Filter directly from the main ranking bar across 6 specialized industrial domains:
  * **Software Development & Coding**: Dedicated software engineering, unit test generation, and bug fixing.
  * **Visual Retouching & Design**: Text-guided image editing, canvas transformation, and creative rendering.
  * **Real-Time Voice & Speech**: Ultra-low latency voice synthesis and speech-to-speech conversion.
  * **DNA & Genomics**: Biological sequence modeling, molecular folding, and biomedical research.
  * **3D Spatial World Models**: Spatial physical world simulation, CAD synthesis, and robotics navigation.
  * **Autonomous Multi-Agent Swarms**: Collaborative multi-agent teams executing complex analytical workflows.
* **Self-Adapting Classification**: Automatically infers domain assignment through a combination of model metadata, modalities, and dominant benchmark scores.

### 4. Side-by-Side Dual Comparator (VS Mode)
* Pin any two models to compare their strengths and weaknesses across 5 orthogonal axes:
  * General Intelligence
  * Software Engineering
  * Autonomous Tool Use
  * Hardware Footprint & Memory Efficiency
  * Inference Speed (Tokens/s)
* Symmetrical dual-bar charts with aligned baselines, winner highlights, and clear visual delta.
* Automated machine fitness verdict assessing whether each contender fits within the user's specific target hardware rig without memory paging or OOM crashes.

### 5. Interactive Pareto Frontier Analysis (`egui_plot`)
* 2D scatter visualization with automatic computation and rendering of the optimal **Pareto Frontier**.
* Easily identifies non-dominated models that offer the highest intelligence for a given memory footprint.
* Ergonomic navigation: bounded viewport preventing loss of view, 1-click recentering, search, and dynamic hover inspection.

### 6. Customizable Hierarchical Data Grid
* **Root Family Grouping**: Consolidates model variants and quantization levels (e.g. 4-bit, 8-bit, full precision) under a single foldable family root, eliminating table clutter.
* **Column Chooser Modal**: Select, order, and toggle visibility for all table columns (rank, model identity, target hardware, footprint, calibrated scores, speed, pricing).
* **Advanced Column Context Menu**: Right-click any column header to sort, hide, or apply advanced filter operators (`contains`, `not contains`, numeric thresholds `>`, `<`, `=`).
* **Persistent Settings**: User display preferences and column layouts are saved automatically in the local database.

### 7. Interactive Onboarding & User Guide
* Built-in interactive walkthrough with spotlights, step-by-step guidance, and real-time UI validation.
* Video auto-play simulation mode explaining core concepts: memory budgeting, local vs cloud cost arbitration, and benchmark interpretation.

### 8. Storage & Zero External Daemons
* Local embedded database (`benchmark.db`) powered by `rusqlite` with bundled SQLite C compilation.
* Zero external background services, containers, or daemon processes required.
* Immutable timestamped snapshots for time-travel comparisons across synchronization dates.

---

## Architecture

The project strictly adheres to **Clean Architecture** and **Inversion of Control (IoC)**:

```text
src/
├── domain/                    # Enterprise business logic, entities & traits (Ports)
│   ├── hardware.rs            # Target hardware profiles & platform thresholds
│   ├── modality.rs            # Functional modalities (Text, Multimodal, Vision, Audio)
│   ├── model.rs               # Entities (ModelBenchmark, BenchmarkScores)
│   ├── scoring.rs             # Multi-source calibration & confidence calculation
│   ├── simulator.rs           # Hardware rig simulator & memory budgeting
│   ├── specialty.rs           # Specialized industrial domains & auto-detection
│   └── traits.rs              # Interfaces: IBenchmarkRepository, IModelFetcher, ISystemDetector
├── infrastructure/            # Concrete adapters (Persistence & Network)
│   ├── fetcher/               # Upstream benchmark APIs & multi-source unifier
│   ├── hardware/              # Native cross-platform system memory detector
│   ├── preferences/           # SQLite persistence for table preferences
│   └── repository/            # SQLite snapshot repository
├── presentation/              # Presentation layer (eframe / egui)
│   ├── app.rs                 # Central GUI controller & navigation
│   ├── filtering.rs           # Multi-dimensional filter state machine
│   ├── theme.rs               # Centralized semantic theme tokens
│   ├── i18n/                  # Dynamic bilingual system (FR / EN)
│   └── components/            # UI components (table, graph, comparator, simulator)
├── lib.rs
└── main.rs                    # Dual-mode entry point (Desktop GUI & Headless CLI)
```

---

## Industrial Engineering Standards

* **Strict File Size Cap**: Every source file in this repository is strictly capped at **$\le 300$ lines of code** to enforce Single Responsibility and high maintainability.
* **Zero Hardcoded Data**: Zero static catalogs in production code. All models originate dynamically from network synchronization or local SQLite snapshots.
* **Zero Missing Glyphs**: Strict typography standard eliminating missing character boxes across custom rendering engines.
* **Movable Dialogs**: All modals and popups are floating and user-repositionable.
* **Test Coverage**: Thorough test suite covering unit logic, BDD scenarios, hardware rig simulation, in-memory repository lifecycles, and UI preference persistence (`cargo test`).

---

## Quick Start

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (version 1.80+ recommended)

### Run Desktop GUI
```powershell
cargo run --release
```

### Headless CLI Synchronization
To synchronize live datasets into the local database without opening the GUI:
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
