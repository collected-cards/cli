use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use colored::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::i18n::t;

fn completion_path(shell: Shell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match shell {
        Shell::Bash => {
            // ~/.local/share/bash-completion/completions/collected
            Some(home.join(".local/share/bash-completion/completions/collected"))
        }
        Shell::Zsh => {
            // ~/.zsh/completions/_collected (common custom fpath)
            Some(home.join(".zsh/completions/_collected"))
        }
        Shell::Fish => {
            // ~/.config/fish/completions/collected.fish
            Some(home.join(".config/fish/completions/collected.fish"))
        }
        _ => None,
    }
}

/// `collected completions <shell>` — Generate and install shell completions
pub fn generate_completions(shell: Shell) -> Result<()> {
    let mut app = crate::Cli::command();
    let app_name = app.get_name().to_string();

    if let Some(path) = completion_path(shell) {
        // Create parent dirs
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate into buffer
        let mut buf = Vec::new();
        generate(shell, &mut app, &app_name, &mut buf);

        // Write to file
        let mut file = fs::File::create(&path)?;
        file.write_all(&buf)?;

        println!("  {} {}", "✅".green(), t("completions.installed"));
        println!("  📁 {}", path.display().to_string().cyan());

        match shell {
            Shell::Bash => {
                println!();
                println!("  {}", "Aktivieren:".dimmed());
                println!("  source {}", path.display());
                println!("  {}", "(Oder neue Shell öffnen)".dimmed());
            }
            Shell::Zsh => {
                println!();
                println!("  {}", "In ~/.zshrc hinzufügen:".dimmed());
                println!("  fpath=(~/.zsh/completions $fpath)");
                println!("  autoload -Uz compinit && compinit");
            }
            Shell::Fish => {
                println!();
                println!("  {}", "(Sofort aktiv in neuer Fish-Shell)".dimmed());
            }
            _ => {}
        }
    } else {
        // Fallback: print to stdout
        generate(shell, &mut app, app_name, &mut std::io::stdout());
        eprintln!();
        eprintln!("  {} Automatische Installation nicht unterstützt für {:?}", "⚠️", shell);
        eprintln!("  Leite die Ausgabe in die passende Datei um.");
    }

    Ok(())
}
