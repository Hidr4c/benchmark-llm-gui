# Changelog

All notable changes to **LLM Benchmark Explorer** are documented in this file.

## [1.2.2] - 2026-11-18

### Added & Improved
- **Direct Windows `.exe` Release Binary**: GitHub Releases now publish standalone `benchmark-llm-gui-windows-x64.exe` directly alongside the compressed `.zip`, Linux `.tar.gz`, and macOS ARM64 `.tar.gz` bundles.
- **Complete Repository Documentation**: Overhauled `README.md` with dynamic GitHub Release badge, accurate Clean Architecture tree, and full `v1.2.x` documentation, added `CHANGELOG.md`, and refined live dynamic source descriptions in `sources_ecosystem`.

## [1.2.1] - 2026-11-18

### Added
- **Unified Column Header Filters & Sorting Across All Tables**: Shared `table_filter_header` component delivering clickable Asc/Desc headers, right-click menus, and inline column search operators (`~`, `=`, `<=`, `>=`, `!~`, `.*`, `!.*`) across Harnesses Catalog, Local Runtimes, and the Simulator Compatible Models table.
- **100% Dynamic Ecosystem Discovery (`awesome_discovery`)**: Replaced all static software and harness repository lists in `RuntimeAdapter` and `HarnessAdapter` with live Markdown catalog parsing (`shvyac/awesome-local-llm`, `rafska/awesome-local-llm`, `janhq/awesome-local-ai`, `bradAGI/awesome-cli-coding-agents`), live GitHub topic search, Hugging Face `llm-speed` runs, and real-time `ungh.cc` metadata enrichment.
- **Live-Reactive Simulator & Embedded Clickable Table**: Removed the manual Apply/Refresh friction and floating modal blocker in favor of instant RAM/hardware updates, uniform clickable Top 4 cards (without redundant Inspect buttons), and an embedded Compatible Models table with reliable full-row click inspection.
- **Cloud API Pareto Refinements**: Added blended 3:1 Cloud API pricing calculation, clean exclusion of unpriced API endpoints (preventing misleading `$0.00` Pareto champions), default logarithmic USD X-axis ticks (`$0.01` to `$50+`), and Speed-axis fallback notice for unpriced media APIs.

### Changed & Fixed
- **Zero Hardcoded Catalogs**: Removed remaining hardcoded family/provider lists via morphological dynamic extraction (`extract_dynamic_family`).
- **Media Benchmarks Isolation**: Dedicated `ImageBench`, `VideoBench`, and `AudioBench` columns fed by VBench, OpenASR, LMArena Media, TTS Arena, and ComfyUI indices without polluting `Open LLM 2`.
- **Self-Updater & Version Indicator**: Replaced static update button with live current version display, startup/refresh GitHub Releases auto-check, and automated archive extraction + binary restart flow.
- **Tied-Row Render Stability**: Fixed frame-to-frame row swapping on tied scores using deterministic total-order comparators.

## [1.2.0] - 2026-11-17

### Added
- Windows GUI subsystem launch (`windows_subsystem = "windows"`), complete FR/EN localization parity, live hardware profiler improvements, and automated release packaging.

## [1.1.0] - 2026-11-05

### Added
- Specialized industrial domains (6 verticals), asynchronous in-process hardware detection, side-by-side VS comparator bars, interactive walkthrough tour, and embedded database time-travel snapshots.

## [1.0.0] - 2026-11-01

### Added
- Initial public desktop release with Rust `eframe`/`egui` UI, embedded SQLite persistence, multi-source score calibration, and multiplatform release automation.
