use std::collections::HashMap;

const VRAM_BENCHMARKS_URL: &str = "https://huggingface.co/datasets/llmbenchio/benchmarks-by-vram/raw/main/benchmarks_by_vram.csv";

#[derive(Debug, Clone, Default)]
pub struct VramBenchmarkRecord {
    pub coding: Option<f32>,
    pub agentic: Option<f32>,
    pub quality: Option<f32>,
    pub tok_s: Option<f32>,
}

pub struct VramBenchmarkAdapter {
    client: reqwest::Client,
}

impl VramBenchmarkAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn fetch_benchmarks(&self) -> Result<HashMap<String, VramBenchmarkRecord>, String> {
        let res = self.client.get(VRAM_BENCHMARKS_URL)
            .send()
            .await
            .map_err(|e| format!("VRAM benchmarks request failed: {e}"))?;

        let text = res.text().await.map_err(|e| format!("VRAM benchmarks body error: {e}"))?;

        let mut map = HashMap::new();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(text.as_bytes());

        for result in rdr.records() {
            if let Ok(record) = result {
                // Headers: vram_gb,hardware_kind,hardware,model,run_count,avg_quality,avg_tok_s,scenario_coding,scenario_agent,...
                if record.len() >= 9 {
                    let model_name = record.get(3).unwrap_or("").trim();
                    let quality = record.get(5).and_then(|s| s.parse::<f32>().ok());
                    let tok_s = record.get(6).and_then(|s| s.parse::<f32>().ok());
                    let coding = record.get(7).and_then(|s| s.parse::<f32>().ok());
                    let agentic = record.get(8).and_then(|s| s.parse::<f32>().ok());

                    if !model_name.is_empty() {
                        let key: String = model_name.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
                        map.insert(key, VramBenchmarkRecord {
                            coding,
                            agentic,
                            quality,
                            tok_s,
                        });
                    }
                }
            }
        }

        Ok(map)
    }
}
