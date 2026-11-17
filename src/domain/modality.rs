use serde::{Deserialize, Serialize};

/// Represents the functional modality and application type of an AI model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModelModality {
    #[default]
    Text,           // Large Language Model (Text generation, reasoning, coding)
    Multimodal,     // Vision-Language (Image understanding, OCR, VLM)
    ImageGen,       // Image generation (Diffusion, Flux, SD, Qwen-Image, Krea...)
    VideoGen,       // Video generation (Wan, LTX, Sora, Kling, Runway...)
    AudioSpeech,    // Audio / Speech / Music (Whisper, TTS, ACE-Step, Stable Audio...)
    ThreeDGen,      // 3D Generation & Spatial Mesh (TRELLIS.2, Hunyuan3D, TripoSplat...)
    Embedding,      // Embeddings & vector representation
}

impl ModelModality {
    pub fn all() -> [Self; 7] {
        [
            Self::Text,
            Self::Multimodal,
            Self::ImageGen,
            Self::VideoGen,
            Self::AudioSpeech,
            Self::ThreeDGen,
            Self::Embedding,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "Text (LLM)",
            Self::Multimodal => "Multimodal (Vision+Text)",
            Self::ImageGen => "Image Generation",
            Self::VideoGen => "Video Generation",
            Self::AudioSpeech => "Audio, Music & Speech",
            Self::ThreeDGen => "3D Models & Meshes",
            Self::Embedding => "Embeddings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Text => "TXT",
            Self::Multimodal => "VIS",
            Self::ImageGen => "IMG",
            Self::VideoGen => "VID",
            Self::AudioSpeech => "AUD",
            Self::ThreeDGen => "3D",
            Self::Embedding => "EMB",
        }
    }

    pub fn tag_short(&self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Multimodal => "VISION",
            Self::ImageGen => "IMAGE",
            Self::VideoGen => "VIDEO",
            Self::AudioSpeech => "AUDIO",
            Self::ThreeDGen => "3D",
            Self::Embedding => "EMBED",
        }
    }

    pub fn badge_color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Text => (100, 180, 255),       // Soft blue
            Self::Multimodal => (255, 170, 50),   // Warm amber
            Self::ImageGen => (255, 105, 180),    // Vibrant pink
            Self::VideoGen => (186, 85, 211),     // Medium purple
            Self::AudioSpeech => (72, 209, 204),  // Turquoise
            Self::ThreeDGen => (255, 140, 80),    // Copper orange
            Self::Embedding => (144, 238, 144),   // Soft green
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Multimodal => "multimodal",
            Self::ImageGen => "image_gen",
            Self::VideoGen => "video_gen",
            Self::AudioSpeech => "audio_speech",
            Self::ThreeDGen => "3d_gen",
            Self::Embedding => "embedding",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "multimodal" | "vision" | "vlm" => Self::Multimodal,
            "image_gen" | "image" | "diffusion" => Self::ImageGen,
            "video_gen" | "video" => Self::VideoGen,
            "audio_speech" | "audio" | "transcription" | "speech" | "music" => Self::AudioSpeech,
            "3d_gen" | "3d" | "mesh" => Self::ThreeDGen,
            "embedding" | "embeddings" | "embed" => Self::Embedding,
            _ => Self::Text,
        }
    }

    pub fn detect(id: &str, name: &str, pipeline_tag: Option<&str>, tags: &[String]) -> Self {
        let id_low = id.to_lowercase();
        let name_low = name.to_lowercase();
        let combined = format!("{id_low} {name_low}");

        if let Some(pt) = pipeline_tag {
            let pt_low = pt.to_lowercase();
            if pt_low.contains("to-3d") { return Self::ThreeDGen; }
            if pt_low.contains("image-to-image") || pt_low.contains("text-to-image") { return Self::ImageGen; }
            if pt_low.contains("text-to-video") || pt_low.contains("image-to-video") { return Self::VideoGen; }
            if pt_low.contains("speech") || pt_low.contains("audio") || pt_low.contains("transcription") { return Self::AudioSpeech; }
            if pt_low.contains("image-text") || pt_low.contains("visual") || pt_low.contains("document-question") { return Self::Multimodal; }
            if pt_low.contains("feature-extraction") || pt_low.contains("sentence-similarity") { return Self::Embedding; }
        }

        if combined.contains("trellis") || combined.contains("hunyuan3d") || combined.contains("hunyuan-3d")
            || combined.contains("triposplat") || combined.contains("triposr") || combined.contains("pixal3d")
            || combined.contains("sam 3d") || combined.contains("moge") || combined.contains("llama-mesh")
            || tags.iter().any(|t| t.eq_ignore_ascii_case("3d") || t.contains("to 3D")) {
            return Self::ThreeDGen;
        }

        if combined.contains("sora") || combined.contains("kling") || combined.contains("runway")
            || combined.contains("cogvideo") || combined.contains("hunyuanvideo") || combined.contains("ltx-2")
            || combined.contains("ltx-video") || combined.contains("wan-2") || combined.contains("wan2")
            || combined.contains("seedance") || combined.contains("pika") || combined.contains("veo")
            || combined.contains("i2v") || combined.contains("t2v") || id_low.contains("gen-3") {
            return Self::VideoGen;
        }

        if combined.contains("flux") || combined.contains("diffusion") || combined.contains("sdxl")
            || combined.contains("banana") || combined.contains("dall-e") || combined.contains("midjourney")
            || combined.contains("imagen") || combined.contains("qwen-image") || combined.contains("qwen image")
            || combined.contains("z-image") || combined.contains("krea-2") || combined.contains("moss-image")
            || combined.contains("recraft") || combined.contains("firered") {
            return Self::ImageGen;
        }

        if combined.contains("whisper") || combined.contains("moonshine") || combined.contains("tts")
            || combined.contains("stt") || combined.contains("bark") || combined.contains("ace-step")
            || combined.contains("stable-audio") || combined.contains("stable audio") || combined.contains("music")
            || combined.contains("kokoro") || combined.contains("chatterbox") || combined.contains("lyria") {
            return Self::AudioSpeech;
        }

        if combined.contains("vision") || id_low.contains("-vl") || name_low.contains(" vl")
            || combined.contains("pixtral") || combined.contains("llava") || combined.contains("minicpm-v")
            || combined.contains("internvl") || combined.contains("paligemma") || combined.contains("florence")
            || combined.contains("cogvlm") || combined.contains("omni") || id_low.contains("4o")
            || id_low.contains("gemini-1.5") || id_low.contains("gemini-2.") || name_low.contains("gemini 2")
            || id_low.contains("gemini-3") || name_low.contains("gemini 3")
            || tags.iter().any(|t| t == "multimodal" || t == "vision") {
            return Self::Multimodal;
        }

        if combined.contains("embedding") || id_low.contains("bge-") || id_low.contains("nomic-embed") {
            return Self::Embedding;
        }

        Self::Text
    }
}
