#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
use std::path::{Path, PathBuf};
use super::split::split_into_chunks;
use super::attrs::strip_leading_inner_attrs;
use super::trait_condense::condense_trait_block;
use super::service_extract::transform_service_impl;

fn classify(lines: &[String]) -> &'static str {
    for l in lines {
        let t = l.trim();
        if t.starts_with("pub trait ") { return "trait"; }
        if t.starts_with("impl<") && t.contains("Service<") { return "service"; }
        if t.starts_with("pub mod ") && t.ends_with('{') { return "mod"; }
    }
    "other"
}

fn split_pub_mod(
    out_dir: &Path, path: &PathBuf, lines: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let decl_start = refs.iter().position(|l| {
        let t = l.trim(); t.starts_with("pub mod ") && t.ends_with('{')
    }).ok_or("pub mod block not found")?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("chunk");
    let raw_body: Vec<&str> =
        refs[decl_start + 1..refs.len().saturating_sub(1)].to_vec();
    let (outer_attrs, body_start) = strip_leading_inner_attrs(&raw_body);
    let body_lines: Vec<&str> = raw_body[body_start..].to_vec();
    let sub_chunks = split_into_chunks(&body_lines, 47);
    let mut sub_inc: Vec<String> = Vec::new();
    for (j, chunk) in sub_chunks.iter().enumerate() {
        let sub_path = out_dir.join(format!("{stem}.sub{j}.rs"));
        write_maybe_split(out_dir, &sub_path, chunk)?;
        sub_inc.push(format!(
            r#"include!(concat!(env!("OUT_DIR"), "/{stem}.sub{j}.rs"));"#
        ));
    }
    let inc_path = out_dir.join(format!("{stem}.include.rs"));
    std::fs::write(&inc_path, sub_inc.join("\n") + "\n")?;
    let prefix: String = refs[..decl_start].iter().map(|l| format!("{l}\n")).collect();
    let attr_block: String = outer_attrs.iter().map(|a| format!("{a}\n")).collect();
    let decl_line = refs[decl_start].trim();
    let wrapper = format!(
        "{prefix}{attr_block}{decl_line}\ninclude!(concat!(env!(\"OUT_DIR\"), \"/{stem}.include.rs\"));\n}}\n"
    );
    std::fs::write(path, wrapper)?;
    Ok(())
}

pub fn write_maybe_split(
    out_dir: &Path, path: &PathBuf, lines: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if lines.len() <= 99 {
        std::fs::write(path, lines.join("\n") + "\n")?;
        return Ok(());
    }
    match classify(lines) {
        "trait" => {
            let condensed = condense_trait_block(lines);
            write_maybe_split(out_dir, path, &condensed)
        }
        "service" => {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("svc");
            let (condensed, handler_incs) =
                transform_service_impl(out_dir, stem, lines)?;
            // Write handler includes into a separate file so sub.rs stays ≤99 lines.
            let handlers_file = format!("{stem}.handlers.rs");
            std::fs::write(
                out_dir.join(&handlers_file),
                handler_incs.join("\n") + "\n",
            )?;
            let inc_line = format!(
                r#"include!(concat!(env!("OUT_DIR"), "/{handlers_file}"));"#
            );
            let mut out: Vec<String> = vec![inc_line];
            out.extend(condensed);
            std::fs::write(path, out.join("\n") + "\n")?;
            Ok(())
        }
        "mod" => split_pub_mod(out_dir, path, lines),
        _ => {
            std::fs::write(path, lines.join("\n") + "\n")?;
            Ok(())
        }
    }
}
