use std::{fs::{self}, path::Path};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod rfc;
mod prompt;
mod gemini;
mod animation;

#[derive(Parser)]
#[command(name = "specforge")]
#[command(about = "RFC explainer and implementation guide (hackathon edition)")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Generate conceptual explanation of an RFC
    Explain {
        /// RFC number (e.g., 9110 for HTTP)
        rfc: u32,
    },
    
    /// Generate implementation guide and skeleton
    Implement {
        /// RFC number (e.g., 9110 for HTTP)
        rfc: u32,
        
        #[arg(long, default_value = "rust")]
        lang: String,
        
        #[arg(long, default_value = "minimal")]
        scope: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Get API key from environment
    let api_key = std::env::var("GEMINI_API_KEY")
        .context("GEMINI_API_KEY not set. Get one at https://makersuite.google.com/app/apikey")?;
    
    match cli.mode {
        Mode::Explain { rfc } => {
            explain(rfc, &api_key).await?;
        }
        Mode::Implement { rfc, lang, scope } => {
            implement(rfc, &lang, &scope, &api_key).await?;
        }
    }
    
    Ok(())
}

async fn explain(rfc_num: u32, api_key: &str) -> Result<()> {
    let md_path = format!("rfc/explain/{}.md", rfc_num);
    let anim_path = format!("rfc/animations/rfc_{}.py", rfc_num);
    
    if Path::new(&md_path).exists() && Path::new(&anim_path).exists() {
        println!("✅ Already present: {} and animation", md_path);
        return Ok(());
    }
    
    eprintln!("📥 Fetching RFC {}...", rfc_num);
    let spec_text = rfc::fetch_and_clean(rfc_num).await?;
    
    eprintln!("📝 Generating explanation and animation...");
    let prompt = prompt::build_explain_prompt(rfc_num, &spec_text);
    
    let response = gemini::call_gemini(&prompt, api_key).await?;
    
    // Extract markdown and manim script
    let (markdown, manim_script) = animation::extract_manim_script(&response)?;
    
    // Save markdown
    fs::create_dir_all("rfc/explain")?;
    fs::write(&md_path, &markdown)?;
    eprintln!("✅ Saved explanation to {}", md_path);
    
    // Render animation
    match animation::render_manim(&manim_script, rfc_num) {
        Ok(video_path) => {
            eprintln!("✅ Animation rendered!");
            eprintln!("📹 Video location: {}", video_path);
            eprintln!("\n💡 To view:");
            eprintln!("   xdg-open \"{}\"", video_path);  // Linux
            eprintln!("   open \"{}\"", video_path);      // macOS
        }
        Err(e) => {
            eprintln!("⚠️  Animation rendering failed: {}", e);
            // Save the script even if rendering fails
            fs::create_dir_all("rfc/animations")?;
            fs::write(&anim_path, &manim_script)?;
            eprintln!("   Manim script saved to {} for manual rendering", anim_path);
        }
    }
    
    Ok(())
}

async fn implement(rfc_num: u32, lang: &str, scope: &str, api_key: &str) -> Result<()> {
    let impl_path = format!("rfc/implement/{}.md", rfc_num);
    
    // Validate inputs
    if Path::new(&impl_path).exists() {
        println!("✅ Already present: {}", impl_path);
        return Ok(());
    }
    
    if !["rust", "python", "go"].contains(&lang) {
        anyhow::bail!("Unsupported language: {}. Use rust, python, or go", lang);
    }
    if !["minimal", "practical"].contains(&scope) {
        anyhow::bail!("Invalid scope: {}. Use minimal or practical", scope);
    }
    
    eprintln!("📥 Fetching RFC {}...", rfc_num);
    let spec_text = rfc::fetch_and_clean(rfc_num).await?;
    
    eprintln!("📝 Generating {} implementation guide (scope: {})...", lang, scope);
    let prompt = prompt::build_implement_prompt(rfc_num, &spec_text, lang, scope);
    
    let response = gemini::call_gemini(&prompt, api_key).await?;
    
    fs::create_dir_all("rfc/implement")?;  // Add this line
    fs::write(&impl_path, &response)?;
    eprintln!("✅ Saved implementation guide to {}", impl_path);
    
    Ok(())
}