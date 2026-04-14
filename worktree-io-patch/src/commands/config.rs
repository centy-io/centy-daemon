use anyhow::{Context, Result};
use clap::Subcommand;
use worktree_io::{config::Config, opener};

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the current configuration
    Show,
    /// Print the configuration file path
    Path,
    /// Write the default configuration to disk
    Init,
    /// Set a configuration value (e.g. editor.command "code .")
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// Open the configuration file in the editor
    Edit,
}

pub fn cmd_config(action: ConfigAction) -> Result<()> {
    // LLVM_COV_EXCL_LINE
    // LLVM_COV_EXCL_START
    match action {
        ConfigAction::Show => {
            let config = Config::load()?;
            let pretty = toml::to_string_pretty(&config)?;
            print!("{pretty}");
        }
        ConfigAction::Path => {
            let path = Config::path()?;
            println!("{}", path.display());
        }
        ConfigAction::Init => {
            let config = Config::default();
            config.save()?;
            println!("Wrote default config to {}", Config::path()?.display());
        }
        ConfigAction::Set { key, value } => {
            let mut config = Config::load()?;
            config.set_value(&key, &value)?;
            config.save()?;
            println!("Set {key} = {value}");
        }
        ConfigAction::Get { key } => {
            let config = Config::load()?;
            println!("{}", config.get_value(&key)?);
        }
        ConfigAction::Edit => {
            let path = Config::path()?;
            if !path.exists() {
                Config::default().save()?;
            }
            let command = Config::load()?
                .editor
                .command
                .or_else(|| std::env::var("EDITOR").ok())
                .context("No editor configured. Run: worktree config set editor.command \"code .\", or set the $EDITOR environment variable")?;
            opener::open_in_editor(&path, &command, false)?;
        }
    }
    Ok(())
    // LLVM_COV_EXCL_STOP
}
