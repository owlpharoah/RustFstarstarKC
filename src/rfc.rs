use anyhow::{Context, Error};
use reqwest;
pub async fn fetch_and_clean(rfc_num: u32) -> Result<String,Error>{
    let url = format!("https://www.rfc-editor.org/rfc/rfc{}.txt", rfc_num);

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).build()?;

    let response = client.get(&url).header("User-Agent", "rfcbit/0.1")
        .send()
        .await
        .context("Failed to fetch RFC")?;

    if response.status() == 404 {
        anyhow::bail!("RFC {} not found", rfc_num);
    }
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }
    
    let raw_text = response.text().await?;
    
    Ok(clean_rfc_text(&raw_text))
}

fn clean_rfc_text(raw: &str) -> String{
    let lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Remove page markers
            if trimmed.starts_with("[Page ") || trimmed.starts_with("RFC ") {
                return false;
            }
            // Remove form feeds
            if line.contains('\x0C') {
                return false;
            }
            if line.contains("................") {
                return false;
            }
            // Keep non-empty lines
            !trimmed.is_empty()
        })
        .collect();
    
    let full_text = lines.join("\n");
    
    //truncate 50k chars
    if full_text.len() > 500_000 {
        eprintln!("RFC truncated to fit context window");
        full_text[..500_000].to_string()
    } else {
        full_text
    }
}
