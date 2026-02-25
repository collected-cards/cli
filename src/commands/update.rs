use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use serde::Deserialize;

use crate::i18n::t;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    body: String,
}

const GITHUB_REPO: &str = "collected-cards/cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// `collected update` — Check for and install updates
pub async fn self_update() -> Result<()> {
    println!("  {}", t("update.checking"));
    
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    
    let response = client
        .get(&url)
        .header("User-Agent", "collected-cli")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch release info: {}", response.status()));
    }
    
    let release: GitHubRelease = response.json().await?;
    let latest_version = release.tag_name.trim_start_matches('v');
    
    println!("  {}: v{}", t("update.current"), CURRENT_VERSION);
    println!("  {}: v{}", t("update.latest"), latest_version);
    
    let is_newer = version_compare::compare(latest_version, CURRENT_VERSION)
        .map(|c| c == version_compare::Cmp::Gt)
        .unwrap_or(false);
    
    if is_newer {
        println!();
        let prompt = format!("v{} {}  [y/N]", latest_version.green(), t("update.available"));
        
        if !Confirm::new().with_prompt(prompt).default(false).interact()? {
            println!("  {}", t("update.cancelled"));
            return Ok(());
        }
        
        println!("  {}", t("update.downloading"));
        
        let status = self_update::backends::github::Update::configure()
            .repo_owner("collected-cards")
            .repo_name("cli")
            .bin_name("collected")
            .show_download_progress(true)
            .current_version(CURRENT_VERSION)
            .build()?
            .update()?;
        
        match &status {
            s if s.uptodate() => {
                println!("  {}", t("update.up_to_date"));
            },
            s => {
                println!("  {} {}", t("update.success"), s.version());
            },
        }
        
    } else {
        println!("  {} ✓", t("update.up_to_date"));
    }
    
    Ok(())
}
