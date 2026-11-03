use std::collections::HashMap;
use serde::Deserialize;

const LEADERBOARD_URL: &str = "https://datasets-server.huggingface.co/rows?dataset=open-llm-leaderboard/contents&config=default&split=train&offset=0&limit=100";

#[derive(Deserialize, Debug, Clone)]
pub struct LeaderboardRow {
    pub fullname: Option<String>,
    #[serde(rename = "Average ⬆️")]
    pub average: Option<f64>,
    #[serde(rename = "#Params (B)")]
    pub params_b: Option<f64>,
    #[serde(rename = "IFEval")]
    pub ifeval: Option<f64>,
    #[serde(rename = "BBH")]
    pub bbh: Option<f64>,
    #[serde(rename = "MATH Lvl 5")]
    pub math: Option<f64>,
    #[serde(rename = "GPQA")]
    pub gpqa: Option<f64>,
    #[serde(rename = "MUSR")]
    pub musr: Option<f64>,
    #[serde(rename = "MMLU-PRO")]
    pub mmlu_pro: Option<f64>,
    #[serde(rename = "MoE")]
    pub moe: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct LeaderboardResponse {
    #[serde(default)]
    rows: Vec<LeaderboardRowContainer>,
}

#[derive(Deserialize, Debug)]
struct LeaderboardRowContainer {
    row: LeaderboardRow,
}

pub struct LeaderboardAdapter {
    client: reqwest::Client,
}

impl LeaderboardAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn fetch_scores(&self) -> Result<HashMap<String, LeaderboardRow>, String> {
        let res = self.client.get(LEADERBOARD_URL)
            .send()
            .await
            .map_err(|e| format!("Leaderboard request failed: {e}"))?;

        let body = res.json::<LeaderboardResponse>()
            .await
            .map_err(|e| format!("Leaderboard parse failed: {e}"))?;

        let mut map = HashMap::new();
        for row_container in body.rows {
            let r = row_container.row;
            if let Some(fullname) = &r.fullname {
                let key: String = fullname.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
                map.insert(key, r);
            }
        }
        Ok(map)
    }
}
