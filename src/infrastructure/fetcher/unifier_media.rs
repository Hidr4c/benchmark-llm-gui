use std::collections::{HashMap, HashSet};
use crate::domain::modality::ModelModality;
use crate::domain::model::{BenchmarkScores, ModelBenchmark};
use crate::domain::specialty::DomainSpecialty;
use super::helpers::{extract_dynamic_family, normalize};
use super::lmarena_adapter::{ArenaTrackScore, LmArenaAdapter};
use super::media_bench_adapter::MediaBenchRecord;

fn strip_media_noise(norm: &str) -> String {
    let mut s = norm.to_string();
    for suffix in [
        "diffusers", "repackaged", "uncensored", "comfyui", "pruned", "gguf", "onnx",
        "t2v", "i2v", "fp8", "bf16", "int8", "14b", "30b", "27b", "12b", "5b", "3b", "2b", "13b", "1b",
    ] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            if stripped.len() >= 4 { s = stripped.to_string(); }
        }
    }
    s
}

/// Enriches unified models with LMArena, VBench, OpenASR, TTS Arena, and ComfyUI catalogs.
pub fn integrate_media_and_arenas(
    unified: &mut Vec<ModelBenchmark>,
    arena_map: &HashMap<String, ArenaTrackScore>,
    tts_map: &HashMap<String, (String, f32, u64)>,
    media_map: &HashMap<String, MediaBenchRecord>,
    comfy_models: Vec<ModelBenchmark>,
) {
    let mut existing_keys = HashSet::new();

    for m in unified.iter_mut() {
        let kn = strip_media_noise(&normalize(&m.name));
        let ki = strip_media_noise(&normalize(&m.id));
        existing_keys.insert(kn.clone());
        existing_keys.insert(ki.clone());

        apply_arena_to_model(m, &kn, &ki, arena_map);
        apply_media_bench_to_model(m, &kn, &ki, media_map);
        apply_tts_to_model(m, &kn, &ki, tts_map);
    }

    for mut cm in comfy_models {
        let kn = strip_media_noise(&normalize(&cm.name.replace(" (ComfyUI)", "")));
        if !existing_keys.contains(&kn)
            && !existing_keys.iter().any(|k| k.len() >= 5 && kn.len() >= 5 && (k == &kn || (k.contains(&kn) && k.len() <= kn.len() + 6)))
        {
            apply_arena_to_model(&mut cm, &kn, &kn, arena_map);
            apply_media_bench_to_model(&mut cm, &kn, &kn, media_map);
            apply_tts_to_model(&mut cm, &kn, &kn, tts_map);
            existing_keys.insert(kn);
            unified.push(cm);
        } else if let Some(ex) = unified.iter_mut().find(|m| strip_media_noise(&normalize(&m.name)) == kn) {
            if !ex.is_cloud && cm.size_gb > 0.1 && (ex.size_gb - 6.5).abs() < 0.01 {
                ex.size_gb = cm.size_gb;
                ex.vram_rec_gb = cm.vram_rec_gb;
            }
        }
    }

    // Standalone TTS Arena leaders -> dedicated AudioBench score (never fake OpenLLM)
    for (key, (disp_name, score, votes)) in tts_map {
        if !existing_keys.contains(key) && !existing_keys.iter().any(|k| k.len() >= 4 && (k.contains(key) || key.contains(k))) {
            existing_keys.insert(key.clone());
            unified.push(ModelBenchmark {
                id: format!("tts-arena/{key}"),
                name: format!("{disp_name} (TTS)"),
                family: extract_dynamic_family(disp_name, disp_name),
                size_gb: 2.2,
                vram_rec_gb: 3.0,
                tps_hardware: Some("TTS Arena Votes".into()),
                scores: BenchmarkScores { audio_bench: Some(*score), average: Some(*score), ..Default::default() },
                downloads: *votes * 1000,
                likes: *votes,
                hf_url: format!("https://huggingface.co/{disp_name}"),
                context_k: 8,
                is_cloud: false,
                release_year: Some(2026),
                modality: ModelModality::AudioSpeech,
                specialty: DomainSpecialty::VoiceSpeech,
                ..Default::default()
            });
        }
    }

    // Standalone VBench Video & OpenASR Speech leaders
    for (key, rec) in media_map {
        if existing_keys.contains(key) || existing_keys.iter().any(|k| k.len() >= 4 && (k == key || (k.contains(key) && k.len() <= key.len() + 6))) {
            continue;
        }
        existing_keys.insert(key.clone());
        let is_open_repo = rec.source_url.contains("huggingface.co") || rec.source_url.contains("github.com");
        let is_cloud = !is_open_repo;
        let (size_gb, vram_rec_gb) = if is_cloud { (0.0, 0.0) } else if rec.modality == ModelModality::AudioSpeech { (1.8, 2.5) } else { (12.0, 14.0) };
        let scores = match rec.modality {
            ModelModality::VideoGen => BenchmarkScores { video_bench: Some(rec.score), average: Some(rec.score), ..Default::default() },
            ModelModality::AudioSpeech => BenchmarkScores { audio_bench: Some(rec.score), average: Some(rec.score), ..Default::default() },
            _ => BenchmarkScores { image_bench: Some(rec.score), average: Some(rec.score), ..Default::default() },
        };
        unified.push(ModelBenchmark {
            id: format!("bench-media/{key}"),
            name: rec.display_name.clone(),
            family: extract_dynamic_family("", &rec.display_name),
            size_gb,
            vram_rec_gb,
            tps_hardware: Some(rec.suite.to_string()),
            scores,
            downloads: 400_000,
            likes: 1_200,
            hf_url: rec.source_url.clone(),
            context_k: 8,
            is_cloud,
            release_year: rec.release_year.or(Some(2026)),
            modality: rec.modality,
            specialty: if rec.modality == ModelModality::AudioSpeech { DomainSpecialty::VoiceSpeech } else { DomainSpecialty::VisionCreative },
            ..Default::default()
        });
    }

    unified.extend(LmArenaAdapter::into_standalone_models(arena_map, &existing_keys));
}

fn fuzzy_find<'a, V>(map: &'a HashMap<String, V>, kn: &str, ki: &str) -> Option<&'a V> {
    map.get(kn).or_else(|| map.get(ki)).or_else(|| {
        map.iter().find_map(|(k, v)| {
            let ks = strip_media_noise(k);
            if ks.len() >= 4 && (kn == ks || ki.ends_with(&ks) || (kn.contains(&ks) && kn.len() <= ks.len() + 8) || (ks.contains(kn) && kn.len() >= 5 && ks.len() <= kn.len() + 6)) {
                Some(v)
            } else {
                None
            }
        })
    })
}

fn apply_arena_to_model(m: &mut ModelBenchmark, kn: &str, ki: &str, arena_map: &HashMap<String, ArenaTrackScore>) {
    if let Some(tr) = fuzzy_find(arena_map, kn, ki) {
        match m.modality {
            ModelModality::ImageGen => if let Some(sc) = tr.score_image.or(tr.score_vision) {
                m.scores.image_bench = Some(m.scores.image_bench.map_or(sc, |o| o.max(sc)));
            },
            ModelModality::VideoGen => if let Some(sc) = tr.score_video {
                m.scores.video_bench = Some(m.scores.video_bench.map_or(sc, |o| o.max(sc)));
            },
            ModelModality::Multimodal => {
                if let Some(sc) = tr.score_vision.or(tr.score_text) {
                    m.scores.open_llm = Some(m.scores.open_llm.map_or(sc, |old| if old > 25.0 { old } else { sc }));
                }
                if let Some(img) = tr.score_image { m.scores.image_bench = Some(img); }
                if let Some(vid) = tr.score_video { m.scores.video_bench = Some(vid); }
            }
            ModelModality::Text => if let Some(sc) = tr.score_text.or(tr.score_vision) {
                m.scores.open_llm = Some(m.scores.open_llm.map_or(sc, |old| if old > 25.0 { old } else { sc }));
            },
            _ => {}
        }
        if matches!(m.modality, ModelModality::Text | ModelModality::Multimodal) {
            if let Some(wb) = tr.score_webdev {
                let scaled_wb = ((wb * 0.92).clamp(25.0, 86.0) * 10.0).round() / 10.0;
                m.scores.coding = Some(m.scores.coding.map_or(scaled_wb, |old| old.max(scaled_wb)));
            }
        }
    }
}

fn apply_media_bench_to_model(m: &mut ModelBenchmark, kn: &str, ki: &str, media_map: &HashMap<String, MediaBenchRecord>) {
    if let Some(rec) = fuzzy_find(media_map, kn, ki) {
        match (m.modality, rec.modality) {
            (ModelModality::VideoGen, ModelModality::VideoGen) => {
                m.scores.video_bench = Some(m.scores.video_bench.map_or(rec.score, |o| o.max(rec.score)));
            }
            (ModelModality::AudioSpeech, ModelModality::AudioSpeech) => {
                m.scores.audio_bench = Some(m.scores.audio_bench.map_or(rec.score, |o| o.max(rec.score)));
            }
            (ModelModality::ImageGen, ModelModality::ImageGen) => {
                m.scores.image_bench = Some(m.scores.image_bench.map_or(rec.score, |o| o.max(rec.score)));
            }
            _ => {}
        }
    }
}

fn apply_tts_to_model(m: &mut ModelBenchmark, kn: &str, ki: &str, tts_map: &HashMap<String, (String, f32, u64)>) {
    if m.modality != ModelModality::AudioSpeech { return; }
    if let Some((_, sc, _)) = fuzzy_find(tts_map, kn, ki) {
        m.scores.audio_bench = Some(m.scores.audio_bench.map_or(*sc, |o| o.max(*sc)));
        m.scores.average = Some(*sc);
        m.scores.real_world = None;
    }
}
