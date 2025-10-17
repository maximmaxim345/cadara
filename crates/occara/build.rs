use walkdir::WalkDir;

fn main() -> miette::Result<()> {
    let build = opencascade_sys::OpenCascadeSource::new().build();

    // Find all cpp files in the cpp directory
    let cpp_files: Vec<_> = WalkDir::new("cpp")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "cpp" || ext == "cc" || ext == "cxx")
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();
    let include_files: Vec<_> = WalkDir::new("include")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "hpp" || ext == "h")
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    // Watch for changes in the cpp and include directories
    for file in &cpp_files {
        println!("cargo:rerun-if-changed={}", file.to_str().unwrap());
    }
    for file in &include_files {
        println!("cargo:rerun-if-changed={}", file.to_str().unwrap());
    }
    for entry in WalkDir::new("include")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
    }

    // Generate cxx bindings
    cxx_build::bridges(["src/ffi.rs"])
        .files(cpp_files)
        .std("c++20")
        .include("include")
        .include(build.include_dir())
        .compile("occara-cxx-bridge");

    println!("cargo:rerun-if-changed=src/ffi.rs");

    build.link();
    Ok(())
}
