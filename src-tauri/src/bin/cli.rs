use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Neural CLI - Local translation powered by Ollama
#[derive(Parser)]
#[command(name = "neural-cli")]
#[command(version = "0.1.0")]
#[command(about = "Fast local translation using Ollama + qwen2.5:3b", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate a file
    Translate {
        /// Input file path
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Source language (e.g., "English", "Japanese", "Chinese")
        #[arg(long, default_value = "English")]
        from: String,

        /// Target language (e.g., "Japanese", "Simplified Chinese")
        #[arg(long)]
        to: String,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Ollama base URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,

        /// Model name
        #[arg(long, default_value = "qwen2.5:3b")]
        model: String,
    },

    /// Check Ollama health status
    Health {
        /// Ollama base URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
}

async fn translate_text(
    client: &Client,
    text: &str,
    from_lang: &str,
    to_lang: &str,
    base_url: &str,
    model: &str,
) -> Result<String> {
    let prompt = format!(
        "Translate the following markdown document from {} to {}. \
        Preserve all markdown formatting, code blocks, links, and structure. \
        Only provide the translated content without explanations:\n\n{}",
        from_lang, to_lang, text
    );

    let body = json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "num_predict": 4096
        }
    });

    let response = client
        .post(&format!("{}/api/generate", base_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .context("Failed to send request to Ollama")?;

    if !response.status().is_success() {
        anyhow::bail!("Ollama API error: {}", response.status());
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .context("Failed to parse Ollama response")?;

    Ok(ollama_response.response.trim().to_string())
}

async fn check_health(client: &Client, base_url: &str) -> Result<()> {
    let response = client
        .get(&format!("{}/api/tags", base_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .context("Failed to connect to Ollama")?;

    if response.status().is_success() {
        println!("✅ Ollama is running at {}", base_url);

        // List available models
        let tags: serde_json::Value = response.json().await?;
        if let Some(models) = tags["models"].as_array() {
            println!("\n📦 Available models:");
            for model in models {
                if let Some(name) = model["name"].as_str() {
                    println!("  - {}", name);
                }
            }
        }
        Ok(())
    } else {
        anyhow::bail!("Ollama health check failed: {}", response.status())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Translate {
            input,
            from,
            to,
            output,
            ollama_url,
            model,
        } => {
            println!("🔄 Translating {} → {}", from, to);
            println!("📄 Input: {}", input.display());
            println!("📝 Output: {}", output.display());
            println!("🤖 Model: {}", model);

            // Read input file
            let content = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;

            println!("\n⏳ Translating... (this may take a few minutes)");

            // Translate
            let translated = translate_text(&client, &content, &from, &to, &ollama_url, &model)
                .await
                .context("Translation failed")?;

            // Write output file
            fs::write(&output, translated)
                .with_context(|| format!("Failed to write output file: {}", output.display()))?;

            println!("✅ Translation complete!");
            println!("📊 Output size: {} bytes", fs::metadata(&output)?.len());
        }

        Commands::Health { ollama_url } => {
            println!("🔍 Checking Ollama health at {}...\n", ollama_url);
            check_health(&client, &ollama_url).await?;
        }
    }

    Ok(())
}
