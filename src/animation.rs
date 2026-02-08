use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn extract_manim_script(response: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = response.split("---MARKDOWN---").collect();
    if parts.len() < 2 {
        anyhow::bail!("Response missing MARKDOWN section");
    }
    
    let rest = parts[1];
    let sections: Vec<&str> = rest.split("---MANIM---").collect();
    if sections.len() < 2 {
        anyhow::bail!("Response missing MANIM section");
    }
    
    let markdown = sections[0].trim().to_string();
    let manim_part = sections[1];
    
    let manim = manim_part
        .split("---END---")
        .next()
        .context("Response missing END marker")?
        .trim()
        .to_string();
    
    Ok((markdown, manim))
}

pub fn render_manim(script: &str, rfc_num: u32) -> Result<String> {
    let script_dir = Path::new("rfc/animations");
    fs::create_dir_all(script_dir)?;
    
    let script_path = script_dir.join(format!("rfc_{}.py", rfc_num));
    
    // Clean up the script if it has markdown code fences
    let cleaned_script = script
        .trim_start_matches("```python")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    
    fs::write(&script_path, cleaned_script)?;
    
    eprintln!("🎬 Rendering animation with Manim...");
    
    // Get absolute path to the script
    let abs_script_path = std::fs::canonicalize(&script_path)
        .context("Failed to get absolute path to script")?;
    
    // Run manim from project root (don't use .current_dir())
    let output = Command::new("manim")
        .arg("-pql") // preview, quality low for faster rendering
        .arg(&abs_script_path)
        .output()
        .context("Failed to run manim. Is it installed? (pip install manim)")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("Manim stderr: {}", stderr);
        eprintln!("Manim stdout: {}", stdout);
        anyhow::bail!("Manim rendering failed");
    }
    
    // Manim creates media/ in the current directory
    let video_dir = format!("media/videos/rfc_{}/480p15", rfc_num);
    
    // Find .mp4 files in that directory
    if let Ok(entries) = std::fs::read_dir(&video_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("mp4") {
                let full_path = std::fs::canonicalize(entry.path())?;
                return Ok(full_path.to_string_lossy().to_string());
            }
        }
    }
    
    // If no video found, return the expected directory
    Ok(video_dir)
}