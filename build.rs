use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let triple = &env::var("TARGET")?;
    let mut target = triple.split("-");
    let arch = target.next().unwrap_or("x86_64");
    target.next();
    let os = target.next().unwrap_or("linux");
    let search_paths = match arch {
        "i686" => &[".", "bin/", "bin/linux32/", "garrysmod/bin/"][..],
        "x86_64" => &[".", "bin/linux64/", "linux64"][..],
        _ => &[][..],
    };
    for search_path in search_paths {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", search_path);
        println!("cargo:warning={:?}", search_path);
    }
    let mut link_search_path = PathBuf::from(&env::var("CARGO_MANIFEST_DIR")?);
    link_search_path.push("lib");
    link_search_path.push(arch);
    link_search_path.push(os);
    println!("cargo:warning={:?}", &link_search_path);
    if !link_search_path.exists() {
        panic!("Unsupported platform");
    }
    println!(
        "cargo:rustc-link-search=native={}",
        link_search_path.to_str().expect("wtf?")
    );
    Ok(())
}
