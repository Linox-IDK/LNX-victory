//! A fingerprint of everything a shader translation depends on in source:
//! the translator crate (`victory-shader/src`) and the cache's own encoding
//! (`src/shader_cache.rs`). The shader cache keys on it instead of on the
//! whole executable, so a rebuild that changes no translator code keeps
//! every cached translation (each rebuild translated all shaders again).

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn main() {
    let shader_src = Path::new("../victory-shader/src");
    let own = Path::new("src/shader_cache.rs");
    println!("cargo:rerun-if-changed={}", shader_src.display());
    println!("cargo:rerun-if-changed={}", own.display());
    println!("cargo:rerun-if-changed=../victory-shader/Cargo.toml");
    let mut list = Vec::new();
    files(shader_src, &mut list);
    list.push(own.to_path_buf());
    list.push(PathBuf::from("../victory-shader/Cargo.toml"));
    list.sort();
    // FNV-1a, 64 bits, over each file's relative path and bytes.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for p in &list {
        eat(p.to_string_lossy().replace('\\', "/").as_bytes());
        eat(&[0]);
        if let Ok(bytes) = std::fs::read(p) {
            // Line endings do not change a translation.
            let text: Vec<u8> = bytes.into_iter().filter(|&b| b != b'\r').collect();
            eat(&text);
        }
        eat(&[0]);
    }
    println!("cargo:rustc-env=VICTORY_SHADER_SOURCE_HASH={h:016x}");
}
