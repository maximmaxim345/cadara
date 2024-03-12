use walkdir::WalkDir;

fn main() {
    let include_dir = opencascade_sys::include_dir();
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
        "src/lib.rs",
        [&std::path::PathBuf::from("include"), &include_dir],
    )
    .build()
    .unwrap();

    autocxx_build
        .flag_if_supported("-std=c++20")
        .files(files)
        .compile("occara-autocxx-bridge");
    println!("cargo:rerun-if-changed=src/lib.rs");

    // Build inline c++ code using the cpp_build crate
    cpp_build::Config::new()
        .flag_if_supported("-std=c++20")
        .include(include_dir)
        .include("include")
        .build("src/lib.rs");
    opencascade_sys::link_opencascade();
}
