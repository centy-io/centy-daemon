use anyhow::Result;
use worktree_io::ttl::Ttl;

pub(super) fn prompt_ttl() -> Result<Option<Ttl>> {
    // LLVM_COV_EXCL_START
    use std::io::{BufRead, Write};
    const PRESETS: &[(&str, &str)] = &[
        ("1 day", "1day"),
        ("7 days", "7days"),
        ("30 days", "30days"),
        ("90 days", "90days"),
    ];
    eprintln!("\nSet workspace TTL (auto-expire old worktrees):");
    for (i, (label, _)) in PRESETS.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, label);
    }
    eprintln!("  {}. Enter a custom duration", PRESETS.len() + 1);
    eprintln!("  0. Skip (no TTL)");
    eprint!("Choice [0]: ");
    std::io::stderr().flush().ok();

    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();

    let choice: usize = if trimmed.is_empty() {
        0
    } else {
        trimmed.parse().unwrap_or(usize::MAX)
    };

    if choice == 0 {
        return Ok(None);
    }
    if choice <= PRESETS.len() {
        return Ok(Some(PRESETS[choice - 1].1.parse()?));
    }
    if choice == PRESETS.len() + 1 {
        eprint!("Enter duration (e.g. \"30days\", \"12hours\"): ");
        std::io::stderr().flush().ok();
        let mut custom = String::new();
        stdin.lock().read_line(&mut custom)?;
        let s = custom.trim();
        if s.is_empty() {
            return Ok(None);
        }
        return Ok(Some(s.parse::<Ttl>().map_err(|e| anyhow::anyhow!("{e}"))?));
    }

    eprintln!("Invalid choice, skipping TTL configuration.");
    Ok(None)
    // LLVM_COV_EXCL_STOP
}
pub(super) fn prompt_editor(detected: &[(&str, &str)]) -> Result<Option<String>> {
    // LLVM_COV_EXCL_START
    use std::io::{BufRead, Write};

    eprintln!("\nSelect your default editor or terminal:");
    for (i, (name, _)) in detected.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, name);
    }
    let custom_idx = detected.len() + 1;
    eprintln!("  {custom_idx}. Enter a custom command");
    eprintln!("  0. Skip (no editor configured)");
    eprint!("Choice [{}]: ", i32::from(!detected.is_empty()));
    std::io::stderr().flush().ok();

    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();

    let choice: usize = if trimmed.is_empty() {
        usize::from(!detected.is_empty())
    } else {
        trimmed.parse().unwrap_or(usize::MAX)
    };

    if choice == 0 {
        return Ok(None);
    }
    if choice <= detected.len() {
        return Ok(Some(detected[choice - 1].1.to_string()));
    }
    if choice == custom_idx {
        eprint!("Enter editor command (e.g. \"hx .\"): ");
        std::io::stderr().flush().ok();
        let mut custom = String::new();
        stdin.lock().read_line(&mut custom)?;
        let cmd = custom.trim().to_string();
        return Ok(if cmd.is_empty() { None } else { Some(cmd) });
    }

    eprintln!("Invalid choice, skipping editor configuration.");
    Ok(None)
    // LLVM_COV_EXCL_STOP
}
