use std::collections::HashSet;
use serde::Deserialize;

const HF_ENDPOINTS: [&str; 16] = [
    "https://huggingface.co/api/models?filter=gguf&sort=downloads&direction=-1&limit=150",
    "https://huggingface.co/api/models?search=qwen4&sort=downloads&direction=-1&limit=15",
    "https://huggingface.co/api/models?search=qwen3.8&sort=downloads&direction=-1&limit=15",
    "https://huggingface.co/api/models?search=qwen3.5&sort=downloads&direction=-1&limit=15",
    "https://huggingface.co/api/models?search=mistral&filter=gguf&sort=downloads&direction=-1&limit=25",
    "https://huggingface.co/api/models?search=deepseek&filter=gguf&sort=downloads&direction=-1&limit=25",
    "https://huggingface.co/api/models?search=llama-3&filter=gguf&sort=downloads&direction=-1&limit=25",
    "https://huggingface.co/api/models?search=smollm2&filter=gguf&limit=15",
    "https://huggingface.co/api/models?search=genomics&filter=gguf&sort=downloads&direction=-1&limit=15",
    "https://huggingface.co/api/models?author=Comfy-Org&sort=downloads&direction=-1&limit=35",
    "https://huggingface.co/api/models?pipeline_tag=text-to-image&sort=downloads&direction=-1&limit=25",
    "https://huggingface.co/api/models?pipeline_tag=text-to-video&sort=downloads&direction=-1&limit=20",
    "https://huggingface.co/api/models?pipeline_tag=image-to-video&sort=downloads&direction=-1&limit=20",
    "https://huggingface.co/api/models?pipeline_tag=image-to-3d&sort=downloads&direction=-1&limit=20",
    "https://huggingface.co/api/models?pipeline_tag=text-to-audio&sort=downloads&direction=-1&limit=15",
    "https://huggingface.co/api/models?pipeline_tag=text-to-speech&sort=downloads&direction=-1&limit=20",
];

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
        let mut seen = HashSet::new();
        let mut all = Vec::new();
        for url_batch in [
            tokio::join!(
                self.fetch_endpoint(HF_ENDPOINTS[0]),
                self.fetch_endpoint(HF_ENDPOINTS[1]),
                self.fetch_endpoint(HF_ENDPOINTS[2]),
                self.fetch_endpoint(HF_ENDPOINTS[3]),
                self.fetch_endpoint(HF_ENDPOINTS[4]),
                self.fetch_endpoint(HF_ENDPOINTS[5]),
                self.fetch_endpoint(HF_ENDPOINTS[6]),
                self.fetch_endpoint(HF_ENDPOINTS[7]),
            ),
            tokio::join!(
                self.fetch_endpoint(HF_ENDPOINTS[8]),
                self.fetch_endpoint(HF_ENDPOINTS[9]),
                self.fetch_endpoint(HF_ENDPOINTS[10]),
                self.fetch_endpoint(HF_ENDPOINTS[11]),
                self.fetch_endpoint(HF_ENDPOINTS[12]),
                self.fetch_endpoint(HF_ENDPOINTS[13]),
                self.fetch_endpoint(HF_ENDPOINTS[14]),
                self.fetch_endpoint(HF_ENDPOINTS[15]),
            ),
        ] {
            let lists = [url_batch.0, url_batch.1, url_batch.2, url_batch.3, url_batch.4, url_batch.5, url_batch.6, url_batch.7];
            for batch in lists {
                for item in batch {
                    if seen.insert(item.id.clone()) {
                        all.push(item);
                    }
                }
            }
        }

        if all.is_empty() {
            return Err("No models retrieved from Hugging Face Hub".into());
        }
        Ok(all)
    }
}
