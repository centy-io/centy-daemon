#[path = "build/attrs.rs"]
mod attrs;
#[path = "build/mod_split.rs"]
mod mod_split;
#[path = "build/service_extract.rs"]
mod service_extract;
#[path = "build/service_transform.rs"]
mod service_transform;
#[path = "build/split.rs"]
mod split;
#[path = "build/trait_condense.rs"]
mod trait_condense;

use mod_split::write_maybe_split;
use split::split_into_chunks;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("centy_descriptor.bin"))
        .compile_protos(
            &[
                "proto/centy/v1/centy.proto",
                "proto/centy/v1/generic_item.proto",
            ],
            &["proto"],
        )?;

    // Split the generated centy.v1.rs into chunks of <=99 lines each so that
    // no single file exceeds the dylint max_lines_per_file limit.
    //
    // max_chunk_size is chosen so that accumulating one more complete item
    // (the largest top-level struct/enum is ~52 lines) still keeps the chunk
    // within 99 lines: max_chunk_size (47) + max_item_size (52) = 99.
    let proto_file = out_dir.join("centy.v1.rs");
    let content = std::fs::read_to_string(&proto_file)?;
    let lines: Vec<&str> = content.lines().collect();
    let chunks = split_into_chunks(&lines, 47);

    let mut include_lines: Vec<String> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let part_path = out_dir.join(format!("centy.v1.part{i}.rs"));
        write_maybe_split(&out_dir, &part_path, chunk)?;
        include_lines.push(format!(
            r#"include!(concat!(env!("OUT_DIR"), "/centy.v1.part{i}.rs"));"#
        ));
    }

    let include_path = out_dir.join("centy.v1.include.rs");
    std::fs::write(&include_path, include_lines.join("\n") + "\n")?;

    Ok(())
}
