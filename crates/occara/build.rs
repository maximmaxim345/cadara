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

    // Generate cxx bridge code and compile it
    // TODO: figure out how to enable more pedantic warnings for c++ code
    cxx_build::bridge("src/lib.rs")
        .files(files)
        .flag_if_supported("-std=c++20")
        .include(&include_dir)
        .include("include")
        .compile("opencascade-cxx-bridge");

    // Build inline c++ code using the cpp_build crate
    cpp_build::Config::new()
        .flag_if_supported("-std=c++20")
        .include(include_dir)
        .include("include")
        .build("src/lib.rs");
    opencascade_sys::link_opencascade();
}
