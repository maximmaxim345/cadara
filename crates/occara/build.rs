use walkdir::WalkDir;

fn main() -> miette::Result<()> {
    let build = opencascade_sys::OpenCascadeSource::new().build();

    let target = std::env::var("TARGET").unwrap_or_default();
    let target_specific_flags = format!("CXXFLAGS_{}", target.replace("-", "_"));

    // Try target-specific flags first, fall back to general CXXFLAGS
    let cxx_flags = std::env::var(&target_specific_flags)
        .or_else(|_| std::env::var("CXXFLAGS"))
        .unwrap_or_default();

    let all_clang_args: Vec<&str> = cxx_flags
        .split_whitespace()
        .chain(std::iter::once("-std=c++20"))
        .collect();

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

    // Generate autocxx bindings
    let mut autocxx_build = autocxx_build::Builder::new(
        "src/ffi.rs",
        [&std::path::PathBuf::from("include"), build.include_dir()],
    )
    .extra_clang_args(&all_clang_args)
    .build()?;

    autocxx_build
        .std("c++20")
        .files(cpp_files)
        .compile("occara-autocxx-bridge");
    println!("cargo:rerun-if-changed=src/ffi.rs");

    build.link();
    Ok(())
}
