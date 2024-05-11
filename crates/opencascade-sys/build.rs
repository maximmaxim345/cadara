use std::time::SystemTime;
use std::{env, fs, path::Path, process::Command};
use walkdir::WalkDir;

const REPOSITORY: &str = "https://github.com/Open-Cascade-SAS/OCCT.git";
const BRANCH: &str = "OCCT-7.8";
const SOURCE_DIR: &str = "occt_source";
const BUILD_DIR: &str = "occt_build";
const LIB_DIR: &str = "occt_lib";
const INCLUDE_DIR: &str = "occt_include";
const OCCT_VERSION_LOCK_FILE: &str = "occt_commit_hash.lock";

fn main() {
    if !is_git_available() {
        panic!("Git is not available, but is required to build OCCT.")
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_windows = target_os == "windows";

    // Currently a Debug build of OpenCASCADE on Windows is Broken.
    // Some of the Problems are:
    // - unresolved linking errors of some __impl__ symbols
    // - INSTALL_DIR_LIB is partly ignored, it installs to occt_libd for some reason
    // - cmake can't find pdb files
    // - with ninja, for some reason /FS option is ignored, failing to lock pdb files
    // I have no idea how to fix this, but as Release build seems to work,
    // so we will always use that on windows.
    let always_build_release = is_windows;

    let profile = if is_windows {
        "release".to_string()
    } else {
        std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string())
    };

    let current_dir = env::current_dir().expect("Failed to retrieve current directory");
    // let out_dir = env::var("OUT_DIR").expect("Failed to retrieve OUT_DIR environment variable");
    let occt_version_lock_path = current_dir.join(OCCT_VERSION_LOCK_FILE);
    let source_path = Path::new(SOURCE_DIR);
    let build_dir = current_dir.join(BUILD_DIR).join(profile);
    let lib_dir = build_dir.join(LIB_DIR);
    let include_dir = build_dir.join(INCLUDE_DIR);
    let build_marker = build_dir.join(".built");

    // Watch for changes, this is very sensitive, but we will later test if a full rebuild is necessary
    println!("cargo:rerun-if-changed={}", source_path.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", build_dir.to_str().unwrap());
    println!(
        "cargo:rerun-if-changed={}",
        occt_version_lock_path.to_str().unwrap()
    );

    // Prepare the source directory
    if !source_path.exists() {
        // Clone the repository if it doesn't exist
        clone_repository(REPOSITORY, BRANCH, source_path).expect("Failed to clone repository");
        // If the build directory already exists, we should remove it, as it might contain old files
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).expect("Failed to remove build directory");
        }
    }
    // If the last_commit file exists, we should use the commit ID from there
    // otherwise, we save the newest commit ID to the file and use that
    if !occt_version_lock_path.exists() {
        fetch_origin(source_path, BRANCH).expect("Failed to fetch latest commit");
        let latest_commit =
            get_latest_commit(source_path, BRANCH).expect("Failed to get latest commit ID");
        fs::write(&occt_version_lock_path, latest_commit).expect("Failed to write last commit ID");
    } else {
        let occt_version = fs::read_to_string(&occt_version_lock_path).unwrap();
        if !is_git_repo_at_commit(source_path, &occt_version).expect("Failed to check commit ID") {
            // If the commit is different, we should fetch and checkout the commit
            fetch_origin(source_path, BRANCH).expect("Failed to fetch latest commit");
            checkout_commit(source_path, &occt_version).expect("Failed to checkout latest commit");
        }
    }

    // To reduce build times, only build OCCT if necessary
    // the cmake crate still has some problems with this (https://github.com/rust-lang/cmake-rs/issues/65),
    // so we have to do it manually
    // TODO: we split opencascade-sys and their bindings into two crates
    // The complex logic here was necessary to avoid rebuilding the library on every build, when rebuilding
    // bindings. this is not necessary anymore, so simplify this build script
    if is_rebuild_required(source_path, &build_marker) {
        let mut config = cmake::Config::new(SOURCE_DIR);

        // More or less minimal configuration for our use case
        config
            .define("BUILD_MODULE_Draw", "OFF")
            .define("BUILD_MODULE_DataExchange", "OFF")
            .define("BUILD_MODULE_ApplicationFramework", "OFF")
            .define("BUILD_MODULE_Visualization", "OFF")
            .define("BUILD_MODULE_DETools", "OFF")
            .define("USE_FREETYPE", "OFF")
            .define("USE_FREEIMAGE", "OFF")
            .define("USE_OPENVR", "OFF")
            .define("USE_OPENGL", "OFF")
            .define("USE_GLES2", "OFF")
            .define("USE_RAPIDJSON", "OFF")
            .define("USE_DRACO", "OFF")
            .define("USE_TK", "OFF")
            .define("USE_TBB", "OFF")
            .define("USE_VTK", "OFF");

        // Set the install directories
        config
            .define("INSTALL_DIR_LIB", LIB_DIR)
            .define("INSTALL_DIR_INCLUDE", INCLUDE_DIR);

        // We build in the source directory of our crate this is not ideomatic, but has following reasons:
        // - Changes to the library can easily be made for development purposes
        // - The build directory stays consistent.
        //   ("cargo build" would build the library, but running "cargo clippy" afterwards would rebuild the library in a different directory again)
        // - The build path stays consistent across clean rebuilds
        //   (sccache would register each compilation as a cache miss if the build directory changes)
        config.out_dir(&build_dir);

        // We only support static linking for now
        config.define("BUILD_LIBRARY_TYPE", "Static");

        // Use Ninja if available
        if is_ninja_available() {
            config.generator("Ninja");
        }

        if always_build_release {
            config.profile("Release");
        }

        // Use sccache if available (for faster rebuilds)
        if is_sccache_available() {
            config.define("CMAKE_C_COMPILER_LAUNCHER", "sccache");
            config.define("CMAKE_CXX_COMPILER_LAUNCHER", "sccache");
        }

        config.build();

        // Update/Touch the build marker to indicate that the build was successful
        fs::write(&build_marker, "").expect("Failed to update build marker");
    }

    // Opencascade is now successfully built, export environment variables,
    // so lib.rs can be used to link against the library
    println!("cargo:rustc-env=OPENCASCADE_LIB_DIR={}", lib_dir.display());
    println!(
        "cargo:rustc-env=OPENCASCADE_INCLUDE_DIR={}",
        include_dir.display()
    );
}

fn execute_git_command(args: &[&str], source_dir: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(source_dir)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn clone_repository(repository: &str, branch: &str, target_dir: &Path) -> Result<(), String> {
    execute_git_command(
        &[
            "clone",
            "-b",
            branch,
            repository,
            target_dir.to_str().ok_or("Invalid target dir")?,
        ],
        Path::new("."),
    )
    .map(|_| ())
}

fn fetch_origin(source_dir: &Path, branch: &str) -> Result<(), String> {
    execute_git_command(&["fetch", "origin", branch], source_dir).map(|_| ())
}

fn checkout_commit(source_dir: &Path, commit: &str) -> Result<(), String> {
    execute_git_command(&["checkout", commit], source_dir).map(|_| ())
}

fn get_latest_commit(source_dir: &Path, branch: &str) -> Result<String, String> {
    execute_git_command(&["rev-parse", &format!("origin/{}", branch)], source_dir)
}

fn get_current_commit(source_dir: &Path) -> Result<String, String> {
    execute_git_command(&["rev-parse", "HEAD"], source_dir)
}

/// Checks if the provided source directory is at the expected commit hash.
fn is_git_repo_at_commit(source_path: &Path, commit: &str) -> Result<bool, String> {
    let current_commit = get_current_commit(source_path)?;
    Ok(current_commit == commit)
}

fn is_git_available() -> bool {
    Command::new("git").arg("--version").status().is_ok()
}

fn is_ninja_available() -> bool {
    Command::new("ninja").arg("--version").status().is_ok()
}

fn is_sccache_available() -> bool {
    Command::new("sccache").arg("--version").status().is_ok()
}

fn find_latest_modification_time(dir_path: &Path) -> Option<SystemTime> {
    WalkDir::new(dir_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        // This will put all files into the cargo watch list, from my testing on linux, this is not required
        // .map(|e| {
        //     // Add to cargo watch list
        //     println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
        //     e
        // })
        .filter_map(|entry| fs::metadata(entry.path()).ok())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
}

/// Check if a rebuild is required, that means if the source files or the build.rs file were modified after the last build
/// This does not check if the source is at the expected commit hash
fn is_rebuild_required(source_path: &Path, build_marker: &Path) -> bool {
    if !source_path.exists() {
        // Always build if the source directory doesn't exist.
        return true;
    }
    if !build_marker.exists() {
        // We did not build yet, so we should build
        return true;
    }
    // The commit is the same, but the user might have modified some source files
    let build_marker_modified = fs::metadata(build_marker)
        .ok()
        .and_then(|metadata| metadata.modified().ok());

    // Find the latest modification time of any file in the source directory, Metadata::modified() depends on the system
    // in the case a file was modified inside the directory, but the directory itself wasn't touched
    let source_modified = find_latest_modification_time(source_path);
    let buildrs_modified = fs::metadata("build.rs")
        .ok()
        .and_then(|metadata| metadata.modified().ok());

    // If the source or build.rs files were modified after the build marker, we should rebuild
    match (source_modified, buildrs_modified, build_marker_modified) {
        (Some(source_modified), Some(buildrs_modified), Some(build_marker_modified)) => {
            source_modified > build_marker_modified || buildrs_modified > build_marker_modified
        }
        _ => {
            // We couldn't get some of the modification times. if in doubt, we should build, cmake will handle the rest
            true
        }
    }
}
