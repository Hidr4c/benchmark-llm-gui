use serde::{Deserialize, Serialize};

/// Represents the functional modality and application type of an AI model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModelModality {
    #[default]
    Text,           // Large Language Model (Text generation, reasoning, coding)
    Multimodal,     // Vision-Language (Image understanding, OCR, VLM)
    ImageGen,       // Image generation (Diffusion, Flux, SD...)
    VideoGen,       // Video generation (Sora, Kling, Runway, CogVideo...)
    AudioSpeech,    // Audio / Speech / Transcription (Whisper, TTS...)
    Embedding,      // Embeddings & vector representation
}

impl ModelModality {
    pub fn all() -> [Self; 6] {
        [
            Self::Text,
            Self::Multimodal,
            Self::ImageGen,
            Self::VideoGen,
            Self::AudioSpeech,
            Self::Embedding,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "📝 Text (LLM)",
            Self::Multimodal => "👁️ Multimodal (Vision+Text)",
            Self::ImageGen => "🎨 Image Generation",
            Self::VideoGen => "🎬 Video Generation",
            Self::AudioSpeech => "🎙️ Audio & Speech",
            Self::Embedding => "🔢 Embeddings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Text => "📝",
            Self::Multimodal => "👁️",
            Self::ImageGen => "🎨",
            Self::VideoGen => "🎬",
            Self::AudioSpeech => "🎙️",
            Self::Embedding => "🔢",
        }
    }

    pub fn tag_short(&self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Multimodal => "VISION",
            Self::ImageGen => "IMAGE",
            Self::VideoGen => "VIDEO",
            Self::AudioSpeech => "AUDIO",
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
            Self::Embedding => "embedding",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "multimodal" | "vision" | "vlm" => Self::Multimodal,
            "image_gen" | "image" | "diffusion" => Self::ImageGen,
            "video_gen" | "video" => Self::VideoGen,
            "audio_speech" | "audio" | "transcription" | "speech" => Self::AudioSpeech,
            "embedding" | "embeddings" | "embed" => Self::Embedding,
            _ => Self::Text,
        }
    }

    pub fn detect(id: &str, name: &str, pipeline_tag: Option<&str>, tags: &[String]) -> Self {
        let id_low = id.to_lowercase();
        let name_low = name.to_lowercase();

        if let Some(pt) = pipeline_tag {
            let pt_low = pt.to_lowercase();
            if pt_low.contains("image-to-image") || pt_low.contains("text-to-image") {
                return Self::ImageGen;
            }
            if pt_low.contains("text-to-video") || pt_low.contains("image-to-video") {
                return Self::VideoGen;
            }
            if pt_low.contains("speech") || pt_low.contains("audio") || pt_low.contains("transcription") {
                return Self::AudioSpeech;
            }
            if pt_low.contains("image-text") || pt_low.contains("visual") || pt_low.contains("document-question") {
                return Self::Multimodal;
            }
            if pt_low.contains("feature-extraction") || pt_low.contains("sentence-similarity") {
                return Self::Embedding;
            }
        }

        if id_low.contains("flux") || id_low.contains("diffusion") || id_low.contains("sdxl")
            || id_low.contains("banana") || name_low.contains("banana") || name_low.contains("flux")
            || id_low.contains("dall-e") || id_low.contains("midjourney") || id_low.contains("imagen") {
            return Self::ImageGen;
        }

        if id_low.contains("sora") || id_low.contains("kling") || id_low.contains("runway")
            || id_low.contains("cogvideo") || id_low.contains("hunyuanvideo") || id_low.contains("ltx-video")
            || name_low.contains("sora") || name_low.contains("kling") || id_low.contains("gen-3") {
            return Self::VideoGen;
        }

        if id_low.contains("whisper") || id_low.contains("moonshine") || id_low.contains("tts")
            || id_low.contains("stt") || id_low.contains("bark") || name_low.contains("whisper") {
            return Self::AudioSpeech;
        }

        if id_low.contains("vision") || id_low.contains("-vl") || id_low.contains("pixtral")
            || id_low.contains("llava") || id_low.contains("minicpm-v") || id_low.contains("internvl")
            || id_low.contains("paligemma") || id_low.contains("florence") || id_low.contains("cogvlm")
            || name_low.contains("vision") || name_low.contains(" vl") || id_low.contains("4o")
            || id_low.contains("gemini-1.5") || id_low.contains("gemini-2.0") || name_low.contains("gemini 2")
            || tags.iter().any(|t| t == "multimodal" || t == "vision") {
            return Self::Multimodal;
        }

        if id_low.contains("embedding") || id_low.contains("bge-") || id_low.contains("nomic-embed") {
            return Self::Embedding;
        }

        Self::Text
    }
}
