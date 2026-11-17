use crate::domain::harness::{normalize_base_model_key, HarnessRun};

const CLINE_TB_CSV_URL: &str = "https://raw.githubusercontent.com/cline/benchmark-results/main/terminal-bench/2026-06-open-weights/summary.csv";
const KILOBENCH_URL: &str = "https://kilo.ai/kilobench";

pub fn extract_site_url(site: &Option<serde_json::Value>) -> String {
    match site {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr.first().and_then(|v| v.as_str()).unwrap_or("https://www.swebench.com/").to_string(),
        _ => "https://www.swebench.com/".to_string(),
    }
}

pub async fn fetch_cline_opencode_runs(client: &reqwest::Client) -> Vec<HarnessRun> {
    let Ok(resp) = client.get(CLINE_TB_CSV_URL).timeout(std::time::Duration::from_secs(10)).send().await else {
        return Vec::new();
    };
    let Ok(text) = resp.text().await else { return Vec::new(); };
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 7 { continue; }
        let group = cols[0].trim();
        let model = cols[1].trim();
        let Ok(rate) = cols[6].trim().parse::<f32>() else { continue; };
        let (harness_name, harness_org) = normalize_csv_agent_group(group);
        out.push(HarnessRun {
            harness_name,
            harness_org,
            model_key: normalize_base_model_key(model),
            model_name: model.to_string(),
            suite: format!("Terminal-Bench ({group})"),
            score_pct: (rate * 1000.0).round() / 10.0,
            cost_usd: None,
            date: "2026-06".to_string(),
            source_url: "https://github.com/cline/benchmark-results".to_string(),
        });
    }
    out
}

pub async fn fetch_kilobench_runs(client: &reqwest::Client) -> Vec<HarnessRun> {
    let Ok(resp) = client.get(KILOBENCH_URL).timeout(std::time::Duration::from_secs(10)).send().await else {
        return Vec::new();
    };
    let Ok(html) = resp.text().await else { return Vec::new(); };
    let mut out = Vec::new();
    for tr in html.split("<tr").skip(1) {
        let end_tr = tr.find("</tr>").unwrap_or(tr.len());
        let row_html = &tr[..end_tr];
        let mut cells = Vec::new();
        for td in row_html.split("<td").skip(1) {
            let Some(gt) = td.find('>') else { continue; };
            let end_td = td.find("</td>").unwrap_or(td.len());
            let clean = strip_html_tags(&td[gt + 1..end_td]);
            if !clean.is_empty() { cells.push(clean); }
        }
        if cells.len() < 3 { continue; }
        let Some(score_col) = cells.iter().find(|c| c.ends_with('%')) else { continue; };
        let Ok(score) = score_col.trim_end_matches('%').trim().parse::<f32>() else { continue; };
        let cost = cells.iter().find(|c| c.starts_with('$')).and_then(|c| c.trim_start_matches('$').parse::<f32>().ok());
        let raw_model = &cells[1];
        let clean_model = raw_model
            .replace("($$$$)", "").replace("($$$)", "").replace("($$)", "").replace("($)", "")
            .trim().to_string();
        if clean_model.is_empty() { continue; }
        let key = normalize_base_model_key(&clean_model);
        if out.iter().any(|r: &HarnessRun| r.model_key == key) { continue; }
        out.push(HarnessRun {
            harness_name: "Kilo Code".to_string(),
            harness_org: "Kilo".to_string(),
            model_key: key,
            model_name: clean_model,
            suite: "KiloBench (Terminal-Bench)".to_string(),
            score_pct: score,
            cost_usd: cost,
            date: "2026".to_string(),
            source_url: KILOBENCH_URL.to_string(),
        });
    }
    out
}

fn normalize_csv_agent_group(group: &str) -> (String, String) {
    let root = group.split(|c: char| matches!(c, '-' | '_' | ' ')).next().unwrap_or(group).trim();
    let mut chars = root.chars();
    let title = match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Agent".to_string(),
    };
    (title.clone(), title)
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
