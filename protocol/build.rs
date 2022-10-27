use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let proto_bin_path = protoc_bin_vendored::protoc_bin_path()?;
    let proto_path = "src/proto";
    println!("cargo:rerun-if-changed={}", proto_path);
    let all_protos = include_all_protos(proto_path)?;
    protobuf_codegen::Codegen::new()
        .protoc_path(&*proto_bin_path)
        .includes(&[proto_path])
        .inputs(all_protos)
        .cargo_out_dir("proto")
        .run_from_script();
    Ok(())
}

fn include_all_protos(p: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut all_protos = vec![];
    let paths = fs::read_dir(p)?;
    for p in paths {
        let p = p?;
        if let Some(ext) = p.path().extension() {
            if ext == "proto" {
                all_protos.push(p.path())
            }
        }
    }
    Ok(all_protos)
}