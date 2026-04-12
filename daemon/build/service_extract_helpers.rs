#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
use super::service_transform::{build_default_fn, build_handler_fn, path_to_fn_name};
use std::path::Path;

pub fn extract_braced(refs: &[&str], open: usize) -> (Vec<String>, usize) {
    let (mut depth, mut body) = (0i32, Vec::new());
    for (i, l) in refs[open..].iter().enumerate() {
        for c in l.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        body.push((*l).to_string());
        if depth == 0 {
            return (body, open + i);
        }
    }
    (body, refs.len() - 1)
}

pub fn build_condensed(
    refs: &[&str],
    call_s: usize,
    match_s: usize,
    arms: &[String],
    def: &str,
) -> Vec<String> {
    let decl = refs[..6]
        .iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .replace("where {", "{");
    let mut out = vec![format!("    {}", decl.trim())];
    for l in &refs[6..call_s] {
        out.push((*l).to_string());
    }
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

pub fn extract_rpc_arms(
    out_dir: &Path,
    stem: &str,
    refs: &[&str],
    match_s: usize,
) -> Result<(Vec<String>, Vec<String>, String), Box<dyn std::error::Error>> {
    let id_stem = stem.replace('.', "_");
    let (mut includes, mut arms, mut i, mut idx) = (vec![], vec![], match_s + 1, 0usize);
    let mut def = String::from("unreachable_rpc(req)");
    while i < refs.len() {
        let t = refs[i].trim();
        if t.starts_with('"') && t.contains("=>") {
            let path = t.split('"').nth(1).unwrap_or("?");
            let fname = path_to_fn_name(path, idx);
            let (body, end) = extract_braced(refs, i);
            let fs = format!("{stem}.rpc{idx}");
            std::fs::write(
                out_dir.join(format!("{fs}.rs")),
                build_handler_fn(&fname, &body).join("\n") + "\n",
            )?;
            includes.push(format!(
                r#"include!(concat!(env!("OUT_DIR"), "/{fs}.rs"));"#
            ));
            arms.push(format!(
                "                \"{path}\" => {fname}(inner,ace,sce,mdms,mems,req),"
            ));
            i = end + 1;
            idx += 1;
        } else if t.starts_with("_ =>") {
            let (body, end) = extract_braced(refs, i);
            let fname = format!("{id_stem}_default_rpc");
            let fs = format!("{stem}.rpc_default");
            std::fs::write(
                out_dir.join(format!("{fs}.rs")),
                build_default_fn(&fname, &body).join("\n") + "\n",
            )?;
            includes.push(format!(
                r#"include!(concat!(env!("OUT_DIR"), "/{fs}.rs"));"#
            ));
            def = format!("{fname}(req)");
            i = end + 1;
        } else {
            if t == "}" {
                break;
            }
            i += 1;
        }
    }
    Ok((arms, includes, def))
}
