use std::collections::HashSet;
use serde::Deserialize;

const HF_TOP_URL: &str = "https://huggingface.co/api/models?filter=gguf&sort=downloads&direction=-1&limit=150";
const HF_MISTRAL_URL: &str = "https://huggingface.co/api/models?search=mistral&filter=gguf&sort=downloads&direction=-1&limit=25";
const HF_DEEPSEEK_URL: &str = "https://huggingface.co/api/models?search=deepseek&filter=gguf&sort=downloads&direction=-1&limit=25";
const HF_LLAMA_URL: &str = "https://huggingface.co/api/models?search=llama-3&filter=gguf&sort=downloads&direction=-1&limit=25";
const HF_LIGHTWEIGHT_URL: &str = "https://huggingface.co/api/models?search=smollm2&filter=gguf&limit=20";
const HF_TINY_URL: &str = "https://huggingface.co/api/models?search=0.5B&filter=gguf&limit=20";

#[derive(Deserialize, Debug, Clone)]
pub struct HfModelItem {
    pub id: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub likes: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pipeline_tag: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
}

pub struct HuggingFaceAdapter {
    client: reqwest::Client,
}

impl HuggingFaceAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    async fn fetch_endpoint(&self, url: &str) -> Vec<HfModelItem> {
        if let Ok(res) = self.client.get(url).send().await {
            if let Ok(items) = res.json::<Vec<HfModelItem>>().await {
                return items;
            }
        }
        Vec::new()
    }

    pub async fn fetch_models(&self) -> Result<Vec<HfModelItem>, String> {
        let (top, mistral, deepseek, llama, light, tiny) = tokio::join!(
            self.fetch_endpoint(HF_TOP_URL),
            self.fetch_endpoint(HF_MISTRAL_URL),
            self.fetch_endpoint(HF_DEEPSEEK_URL),
            self.fetch_endpoint(HF_LLAMA_URL),
            self.fetch_endpoint(HF_LIGHTWEIGHT_URL),
            self.fetch_endpoint(HF_TINY_URL),
        );

        let mut seen = HashSet::new();
        let mut all = Vec::new();

        for item in top.into_iter().chain(mistral).chain(deepseek).chain(llama).chain(light).chain(tiny) {
            if seen.insert(item.id.clone()) {
                all.push(item);
            }
        }

        if all.is_empty() {
            return Err("No GGUF models retrieved from Hugging Face Hub".into());
        }

        Ok(all)
    }
}
