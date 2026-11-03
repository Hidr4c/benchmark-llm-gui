use serde::{Deserialize, Serialize};

/// Represents an industrial or specialized business domain where a foundation model excels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DomainSpecialty {
    #[default]
    General,        // General LLM (Chat, Reasoning, Writing, General Knowledge)
    GenomicsBio,    // Genomics, DNA sequencing, Biology, CRISPR (e.g. Carbon-X, Evo, ESM)
    VisionCreative, // Creative Visual Retouching, Image-to-Image (e.g. Fibo 2.0, Flux)
    WorldModel3D,   // 3D Spatial World Models, Physical Simulation, CAD (e.g. Sky 001, Genie)
    VoiceSpeech,    // Real-Time Speech-to-Speech, Lifelike Voice Cloning (e.g. Dessa, Moshi)
    MultiAgent,     // Autonomous Multi-Agent Swarms & Automated Scientific Discovery
    SoftwareDev,    // Dedicated Software Engineering & Deep Code Reasoning (e.g. Qwen-Coder)
}

impl DomainSpecialty {
    pub fn all() -> [Self; 7] {
        [
            Self::General,
            Self::GenomicsBio,
            Self::VisionCreative,
            Self::WorldModel3D,
            Self::VoiceSpeech,
            Self::MultiAgent,
            Self::SoftwareDev,
        ]
    }

    pub fn label_fr(&self) -> &'static str {
        match self {
            Self::General => "Généraliste & Raisonnement",
            Self::GenomicsBio => "ADN & Génomique (Santé)",
            Self::VisionCreative => "Retouche Visuelle & Graphisme",
            Self::WorldModel3D => "World Models & 3D / CAO",
            Self::VoiceSpeech => "Voix & Speech-to-Speech",
            Self::MultiAgent => "Swarms d'Agents Autonomes",
            Self::SoftwareDev => "Développement & Code",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            Self::General => "General & Reasoning",
            Self::GenomicsBio => "DNA & Genomics (Bio)",
            Self::VisionCreative => "Visual Retouching & Design",
            Self::WorldModel3D => "World Models & 3D / CAD",
            Self::VoiceSpeech => "Voice & Speech-to-Speech",
            Self::MultiAgent => "Autonomous Agent Swarms",
            Self::SoftwareDev => "Software Dev & Coding",
        }
    }

    pub fn badge_text(&self) -> &'static str {
        match self {
            Self::General => "GEN",
            Self::GenomicsBio => "ADN",
            Self::VisionCreative => "Vision",
            Self::WorldModel3D => "3D",
            Self::VoiceSpeech => "Voix",
            Self::MultiAgent => "Agents",
            Self::SoftwareDev => "Code",
        }
    }

    pub fn badge_color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::General => (140, 160, 190),       // Muted slate
            Self::GenomicsBio => (0, 220, 130),     // Vibrant emerald green
            Self::VisionCreative => (255, 105, 180),// Creative pink
            Self::WorldModel3D => (170, 110, 255),  // Futuristic purple
            Self::VoiceSpeech => (0, 215, 235),     // Electric cyan
            Self::MultiAgent => (255, 180, 40),     // Tech amber
            Self::SoftwareDev => (70, 160, 255),    // Dev blue
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::GenomicsBio => "genomics_bio",
            Self::VisionCreative => "vision_creative",
            Self::WorldModel3D => "world_model_3d",
            Self::VoiceSpeech => "voice_speech",
            Self::MultiAgent => "multi_agent",
            Self::SoftwareDev => "software_dev",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "genomics_bio" | "bio" | "dna" | "adn" | "genomics" => Self::GenomicsBio,
            "vision_creative" | "creative" | "vision" | "image_edit" => Self::VisionCreative,
            "world_model_3d" | "3d" | "world_model" | "cad" => Self::WorldModel3D,
            "voice_speech" | "voice" | "speech" | "audio" => Self::VoiceSpeech,
            "multi_agent" | "swarm" | "agent" | "agents" => Self::MultiAgent,
            "software_dev" | "code" | "coding" | "swe" => Self::SoftwareDev,
            _ => Self::General,
        }
    }

    pub fn detect_with_context(
        id: &str,
        name: &str,
        pipeline_tag: Option<&str>,
        tags: &[String],
        desc: &str,
        scores: Option<&crate::domain::model::BenchmarkScores>,
        modality: crate::domain::modality::ModelModality,
    ) -> Self {
        let text_lower = format!("{} {} {} {}", id, name, pipeline_tag.unwrap_or(""), desc).to_lowercase();

        // 1. Multi-Agent Swarms & Autonomous Workflows (Takes precedence over domain application)
        if text_lower.contains("agent-swarm") || text_lower.contains("multi-agent")
            || text_lower.contains("swarm") || text_lower.contains("crewai") || text_lower.contains("autogen")
            || tags.iter().any(|t| t == "multi-agent" || t == "swarm" || t == "agents")
            || (text_lower.contains("agent") && (text_lower.contains("multi") || text_lower.contains("team") || text_lower.contains("swarm") || text_lower.contains("framework"))) {
            return Self::MultiAgent;
        }

        // 2. Genomics & DNA / Biological Foundation Models
        if text_lower.contains("dna") || text_lower.contains("adn") || text_lower.contains("genomic")
            || text_lower.contains("crispr") || text_lower.contains("protein") || text_lower.contains("biology")
            || text_lower.contains("biomedical") || text_lower.contains("medical") || text_lower.contains("clinical")
            || text_lower.contains("biomed") || text_lower.contains("nucleotide")
            || tags.iter().any(|t| t == "dna" || t == "genomics" || t == "biology" || t == "crispr" || t == "medical" || t == "biomedical") {
            return Self::GenomicsBio;
        }

        // 3. Visual Editing, Image-to-Image & Retouching
        if text_lower.contains("retouch") || text_lower.contains("image-to-image")
            || text_lower.contains("inpainting") || text_lower.contains("outpainting") || text_lower.contains("flux")
            || text_lower.contains("diffusion") || text_lower.contains("sdxl") || text_lower.contains("stable-diffusion")
            || (text_lower.contains("image") && !text_lower.contains("embedding"))
            || (pipeline_tag.map_or(false, |p| p.contains("image-to-image") || p.contains("text-to-image"))
                && (text_lower.contains("edit") || text_lower.contains("photo") || text_lower.contains("art"))) {
            return Self::VisionCreative;
        }

        // 4. 3D World Models & Physical CAD / Engineering
        if text_lower.contains("world model") || text_lower.contains("world-model")
            || text_lower.contains("3d-generation") || text_lower.contains("mesh")
            || text_lower.contains("cad") || text_lower.contains("point-cloud") || text_lower.contains("spatial-ai")
            || tags.iter().any(|t| t == "3d" || t == "world-model" || t == "cad" || t == "spatial") {
            return Self::WorldModel3D;
        }

        // 5. Voice, Speech-to-Speech & Ultra-Lifelike TTS
        if text_lower.contains("speech-to-speech") || text_lower.contains("voice-clone")
            || text_lower.contains("voice-conversion") || text_lower.contains("whisper")
            || text_lower.contains("voice") || text_lower.contains("speech")
            || text_lower.contains("tts") || text_lower.contains("stt") || text_lower.contains("audio")
            || pipeline_tag.map_or(false, |p| p.contains("audio-to-audio") || p.contains("voice-conversion") || p.contains("automatic-speech-recognition")) {
            return Self::VoiceSpeech;
        }

        // 6. Dedicated Software Engineering & Deep Coding
        if text_lower.contains("coder") || text_lower.contains("swe-bench") || text_lower.contains("starcoder")
            || text_lower.contains("codestral") || text_lower.contains("deepseek-coder")
            || (text_lower.contains("code") && !text_lower.contains("tokenizer") && !text_lower.contains("unicode")) {
            return Self::SoftwareDev;
        }

        // 7. Modality-based fallback when tags are absent
        match modality {
            crate::domain::modality::ModelModality::Multimodal | crate::domain::modality::ModelModality::ImageGen => {
                return Self::VisionCreative;
            }
            crate::domain::modality::ModelModality::AudioSpeech => {
                return Self::VoiceSpeech;
            }
            _ => {}
        }

        // 8. Self-adapting fallback using benchmark scores
        if let Some(sc) = scores {
            let coding_val = sc.coding.unwrap_or(0.0);
            let agentic_val = sc.agentic.unwrap_or(0.0);
            let open_llm_val = sc.open_llm.unwrap_or(0.0);

            if coding_val > 70.0 && (coding_val > open_llm_val + 10.0 || open_llm_val == 0.0) {
                return Self::SoftwareDev;
            }
            if agentic_val > 70.0 && (agentic_val > open_llm_val + 10.0 || open_llm_val == 0.0) {
                return Self::MultiAgent;
            }
        }

        Self::General
    }

    pub fn detect(id: &str, name: &str, pipeline_tag: Option<&str>, tags: &[String], desc: &str) -> Self {
        Self::detect_with_context(id, name, pipeline_tag, tags, desc, None, crate::domain::modality::ModelModality::Text)
    }
}