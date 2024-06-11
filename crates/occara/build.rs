use walkdir::WalkDir;

fn main() -> miette::Result<()> {
    let build = opencascade_sys::OpenCascadeSource::new().build();
    // Find all cpp files in the cpp directory
    let files: Vec<_> = WalkDir::new("cpp")
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

    // Watch for changes in the cpp and include directories
    println!("cargo:rerun-if-changed=cpp");
    println!("cargo:rerun-if-changed=include");
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.to_str().unwrap());
    }
    for entry in WalkDir::new("include")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
    }

    // Generate autocxx bindings
    let mut autocxx_build = autocxx_build::Builder::new(
        "src/ffi.rs",
        [&std::path::PathBuf::from("include"), build.include_dir()],
    )
    .extra_clang_args(&["-std=c++20"])
    .build()?;

    autocxx_build
        .std("c++20")
        .files(files)
        .compile("occara-autocxx-bridge");
    println!("cargo:rerun-if-changed=src/ffi.rs");

    build.link();
    Ok(())
}
