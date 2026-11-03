use std::collections::HashMap;

const SPEED_CSV_URL: &str = "https://huggingface.co/datasets/llmspeed/llm-speed-benchmarks/raw/main/runs.csv";

#[derive(Debug, Clone)]
pub struct SpeedRecord {
    pub tps: f32,
    pub accelerator: String,
}

pub struct SpeedBenchmarkAdapter {
    client: reqwest::Client,
}

impl SpeedBenchmarkAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn fetch_speeds(&self) -> Result<HashMap<String, SpeedRecord>, String> {
        let res = self.client.get(SPEED_CSV_URL)
            .send()
            .await
            .map_err(|e| format!("Speed CSV request failed: {e}"))?;

        let text = res.text().await.map_err(|e| format!("Speed CSV read failed: {e}"))?;
        let mut raw_groups: HashMap<(String, String), Vec<f32>> = HashMap::new();

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(text.as_bytes());

        for result in rdr.records() {
            if let Ok(record) = result {
                if record.len() >= 8 {
                    let model_name = &record[3];
                    let accelerator = &record[6];
                    if let Ok(tps) = record[7].parse::<f32>() {
                        if tps > 0.0 && tps < 500.0 {
                            let key: String = model_name.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
                            let clean_acc = Self::clean_accelerator(accelerator);
                            raw_groups.entry((key, clean_acc)).or_default().push(tps);
                        }
                    }
                }
            }
        }

        // Compute median TPS for each (model, accelerator) pair to eliminate outlier maximums
        let mut model_acc_medians: HashMap<String, Vec<(String, f32)>> = HashMap::new();
        for ((m_key, acc), mut tps_list) in raw_groups {
            tps_list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median_tps = tps_list[tps_list.len() / 2];
            model_acc_medians.entry(m_key).or_default().push((acc, median_tps));
        }

        // For each model, select the most representative domestic / standard hardware
        let mut map: HashMap<String, SpeedRecord> = HashMap::new();
        for (m_key, mut acc_list) in model_acc_medians {
            // Sort accelerators by preference: domestic discrete GPUs first, then Apple Silicon, then others
            acc_list.sort_by_key(|(acc, _)| {
                if acc.contains("4090") { 1 }
                else if acc.contains("5090") { 2 }
                else if acc.contains("4080") { 3 }
                else if acc.contains("4070") { 4 }
                else if acc.contains("Apple") { 5 }
                else { 10 }
            });

            if let Some((best_acc, median_tps)) = acc_list.into_iter().next() {
                map.insert(m_key, SpeedRecord {
                    tps: median_tps,
                    accelerator: best_acc,
                });
            }
        }

        Ok(map)
    }

    fn clean_accelerator(acc: &str) -> String {
        if acc.contains("5090") {
            "RTX 5090 (32GB)".to_string()
        } else if acc.contains("4090") {
            "RTX 4090 (24GB)".to_string()
        } else if acc.contains("3090") {
            "RTX 3090 (24GB)".to_string()
        } else if acc.contains("4080") {
            "RTX 4080 (16GB)".to_string()
        } else if acc.contains("4070") {
            "RTX 4070 (12GB)".to_string()
        } else if acc.contains("Apple") || acc.contains("M3") || acc.contains("M4") {
            "Apple Silicon".to_string()
        } else {
            acc.split('+').next().unwrap_or(acc).trim().to_string()
        }
    }
}
