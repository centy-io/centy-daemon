#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
use std::path::Path;
use super::service_transform::{build_default_fn, build_handler_fn, path_to_fn_name};

fn extract_braced(refs: &[&str], open: usize) -> (Vec<String>, usize) {
    let (mut depth, mut body) = (0i32, Vec::new());
    for (i, l) in refs[open..].iter().enumerate() {
        for c in l.chars() { match c { '{' => depth+=1, '}' => depth-=1, _=>{} } }
        body.push((*l).to_string());
        if depth == 0 { return (body, open + i); }
    }
    (body, refs.len() - 1)
}

fn condensed_decl(refs: &[&str]) -> String {
    refs[..6].iter().map(|l| l.trim()).collect::<Vec<_>>()
        .join(" ").replace("where {", "{")
}

fn build_condensed(
    refs: &[&str], call_s: usize, match_s: usize, arms: &[String], def: &str,
) -> Vec<String> {
    let mut out = vec![format!("    {}", condensed_decl(refs).trim())];
    for l in &refs[6..call_s] { out.push((*l).to_string()); }
    out.push(format!("        {}", refs[call_s].trim()));
    out.push("            let (ace,sce,mdms,mems,inner) = (self.accept_compression_encodings,self.send_compression_encodings,self.max_decoding_message_size,self.max_encoding_message_size,self.inner.clone());".to_string());
    out.push(format!("            {}", refs[match_s].trim()));
    out.extend_from_slice(arms);
    out.push(format!("                _ => {def},"));
    out.push("            }".to_string());
    out.push("        }".to_string());
    out.push("    }".to_string());
    out
}

/// Transform an oversized `impl<T, B> Service<...>` block.
/// Writes helper files for each match arm.
/// Returns (condensed_impl_lines, include_lines).
pub fn transform_service_impl(
    out_dir: &Path,
    stem: &str,
    lines: &[String],
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let id_stem = stem.replace('.', "_");
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let call_s = match refs.iter().position(|l| l.trim().starts_with("fn call(")) {
        Some(i) => i, None => return Ok((lines.to_vec(), vec![])),
    };
    let match_s = match refs[call_s..].iter().position(|l| l.trim().starts_with("match req")) {
        Some(i) => call_s + i, None => return Ok((lines.to_vec(), vec![])),
    };
    let (mut includes, mut arms, mut i, mut idx) = (vec![], vec![], match_s + 1, 0usize);
    let mut def = String::from("unreachable_rpc(req)");
    while i < refs.len() {
        let t = refs[i].trim();
        if t.starts_with('"') && t.contains("=>") {
            let path = t.split('"').nth(1).unwrap_or("?");
            let fname = path_to_fn_name(path, idx);
            let (body, end) = extract_braced(&refs, i);
            let fs = format!("{stem}.rpc{idx}");
            std::fs::write(out_dir.join(format!("{fs}.rs")),
                build_handler_fn(&fname, &body).join("\n") + "\n")?;
            includes.push(format!(r#"include!(concat!(env!("OUT_DIR"), "/{fs}.rs"));"#));
            arms.push(format!("                \"{path}\" => {fname}(inner,ace,sce,mdms,mems,req),"));
            i = end + 1; idx += 1;
        } else if t.starts_with("_ =>") {
            let (body, end) = extract_braced(&refs, i);
            let fname = format!("{id_stem}_default_rpc");
            let fs = format!("{stem}.rpc_default");
            std::fs::write(out_dir.join(format!("{fs}.rs")),
                build_default_fn(&fname, &body).join("\n") + "\n")?;
            includes.push(format!(r#"include!(concat!(env!("OUT_DIR"), "/{fs}.rs"));"#));
            def = format!("{fname}(req)");
            i = end + 1;
        } else { if t == "}" { break; } i += 1; }
    }
    Ok((build_condensed(&refs, call_s, match_s, &arms, &def), includes))
}
