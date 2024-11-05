//! # opencascade-sys
//!
//! This is a helper library to compile and link the ``OpenCASCADE`` libraries.
//! If you want use ``OpenCASCADE``, use the high level crate 'occara' instead.
//!
//! This crate is intended to be used as a build dependency for the 'occara' crate.

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, fs, path::Path, process::Command};
use walkdir::WalkDir;

const REPOSITORY: &str = "https://github.com/maximmaxim345/cadara-occt.git";
const BRANCH: &str = "OCCT-7.8-cadara";
const OPENCASCADE_DIR_NAME: &str = "opencascade-sys";
const LIB_DIR: &str = "occt_lib";
const INCLUDE_DIR: &str = "occt_include";
const OCCT_VERSION_LOCK_FILE: &str = "occt_commit_hash.lock";

pub struct OpenCascadeSource {
    profile: Option<String>,
}

impl OpenCascadeSource {
    pub fn new() -> Self {
        // Currently a Debug build of OpenCASCADE on Windows is Broken.
        // Some of the Problems are:
        // - unresolved linking errors of some __impl__ symbols
        // - INSTALL_DIR_LIB is partly ignored, it installs to occt_libd for some reason
        // - cmake can't find pdb files
        // - with ninja, for some reason /FS option is ignored, failing to lock pdb files
        // I have no idea how to fix this, but as Release build seems to work,
        // so we will always use that on windows.
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let is_windows = target_os == "windows";
        Self {
            profile: if is_windows {
                Some("Release".to_string())
            } else {
                None
            },
        }
    }

    pub fn build(self) -> OpenCascadeBuild {
        if !is_git_available() {
            panic!("Git is not available, but is required to build OCCT.")
        }

        let current_dir = env::current_dir().expect("Failed to retrieve current directory");
        let cargo_target_dir =
            get_cargo_native_target_dir().expect("target dir could not be determined");

        let occt_version_lock_path = current_dir.join(OCCT_VERSION_LOCK_FILE);

        let occt_dir = &cargo_target_dir.join(OPENCASCADE_DIR_NAME);
        let source_path = &occt_dir.join("source");

        let mut config = cmake::Config::new(source_path);

        let build_dir = if std::env::var("TARGET").unwrap() == std::env::var("HOST").unwrap() {
            // Native build
            occt_dir.join(format!("build-{}", config.get_profile()))
        } else {
            // Cross compilation
            let target = std::env::var("TARGET").unwrap();
            occt_dir.join(format!("build-{}-{}", config.get_profile(), target))
        };
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

        download_source(source_path, occt_dir, &occt_version_lock_path);

        // To reduce build times, only build OCCT if necessary
        // the cmake crate still has some problems with this (https://github.com/rust-lang/cmake-rs/issues/65),
        // so we have to do it manually
        // TODO: we split opencascade-sys and their bindings into two crates
        // The complex logic here was necessary to avoid rebuilding the library on every build, when rebuilding
        // bindings. this is not necessary anymore, so simplify this build script
        if is_rebuild_required(source_path, &build_marker) {
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

            // We build in the target/opencascade-sys directory for following reasons:
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

            if let Some(p) = self.profile {
                config.profile(&p);
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

        // // Opencascade is now successfully built, export environment variables,
        // // so lib.rs can be used to link against the library
        OpenCascadeBuild {
            lib_dir,
            include_dir,
        }
    }
}

impl Default for OpenCascadeSource {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenCascadeBuild {
    include_dir: PathBuf,
    lib_dir: PathBuf,
}

impl OpenCascadeBuild {
    /// Link the ``OpenCASCADE`` libraries against the current build.
    ///
    /// This function should be called from the build.rs file of the crate that depends on this crate.
    /// It will link the ``OpenCASCADE`` libraries statically against the current build, using the
    /// cargo:rustc-link-search=native and cargo:rustc-link-lib=static directives.
    ///
    /// Due to the way the linker works, this function must be called after linking any other
    /// libraries that depend on ``OpenCASCADE``.
    pub fn link(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        // I dont't know why this order works, but it does, so I'm not going to mess with it for now
        let lib_linking_order = vec![
            "TKBO",
            "TKBool",
            "TKBRep",
            "TKFeat",
            "TKFillet",
            "TKG2d",
            "TKG3d",
            "TKGeomAlgo",
            "TKGeomBase",
            "TKHLR",
            "TKMath",
            "TKMesh",
            "TKOffset",
            "TKPrim",
            "TKShHealing",
            "TKTopAlgo",
            "TKernel",
        ];

        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

        for lib in lib_linking_order {
            if target_os == "windows" {
                // I don't know why this is needed, but it is
                println!("cargo:rustc-link-lib=static:-whole-archive={lib}");
            } else {
                println!("cargo:rustc-link-lib=static={lib}");
            }
        }

        if target_os == "windows" {
            // Also link with the user32 library, which is for some reason needed
            println!("cargo:rustc-link-lib=user32");
        }
    }

    /// Returns the path to the ``OpenCASCADE`` library directory.
    ///
    /// This function returns the system library directory for the ``OpenCASCADE`` libraries.
    /// This director contains the build static libraries for the ``OpenCASCADE`` libraries.
    /// For simple linking, use the `link_opencascade` function, which will automatically link the
    /// ``OpenCASCADE`` libraries in the correct order.
    pub fn lib_dir(&self) -> &std::path::PathBuf {
        &self.lib_dir
    }

    /// Returns the path to the ``OpenCASCADE`` include directory.
    ///
    /// This function returns the system include directory for the ``OpenCASCADE`` libraries. It should be
    /// used to include the ``OpenCASCADE`` headers in the build process, for example when generating
    /// bindings to the ``OpenCASCADE`` libraries.
    pub fn include_dir(&self) -> &std::path::PathBuf {
        &self.include_dir
    }
}

// based on https://github.com/rust-lang/cargo/issues/9661#issuecomment-1722358176
fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let profile = std::env::var("PROFILE")?;
    let mut target_dir = None;
    let mut sub_path = out_dir.as_path();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent.parent().ok_or("not found")?);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.ok_or("not found")?;
    Ok(target_dir.to_path_buf())
}

fn get_cargo_native_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let dir = get_cargo_target_dir()?;
    let file_name = dir
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or("Invalid file name")?;
    let target = std::env::var("TARGET")?;
    if file_name == target {
        Ok(dir
            .parent()
            .ok_or("No parent directory found")?
            .to_path_buf())
    } else {
        Ok(dir)
    }
}

fn delete_build_dirs(path: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |s| s.starts_with("build-"))
        {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

fn download_source(
    source_path: &Path,
    build_subdirs: &Path,
    occt_version_lock_path: &std::path::PathBuf,
) {
    let (mut file, exists) = File::open(occt_version_lock_path)
        .map(|f| (f, true))
        .or_else(|_| File::create(occt_version_lock_path).map(|f| (f, false)))
        .unwrap();
    // Prepare the source directory
    if !source_path.exists() {
        // Clone the repository if it doesn't exist
        clone_repository(REPOSITORY, BRANCH, source_path).expect("Failed to clone repository");
        // If build directories already exists, we should remove them, as they might contain old files
        delete_build_dirs(build_subdirs).unwrap();
    } else if get_remote_url(source_path).map_or(true, |url| url != REPOSITORY)
        || get_current_commit(source_path).is_err()
    {
        // Either something failed, or the url has changed
        fs::remove_dir_all(source_path).expect("error deleting source code for redownload");
        download_source(source_path, build_subdirs, occt_version_lock_path);
        return; // retry
    }
    // If the last_commit file exists, we should use the commit ID from there
    // otherwise, we save the newest commit ID to the file and use that
    if !exists {
        fetch_origin(source_path, BRANCH).expect("Failed to fetch latest commit");
        let latest_commit =
            get_latest_commit(source_path, BRANCH).expect("Failed to get latest commit ID");
        file.write_all(latest_commit.as_bytes())
            .expect("Failed to write last commit ID");
    } else {
        let mut occt_version = String::new();
        file.read_to_string(&mut occt_version)
            .expect("Failed to read last commit ID");
        if !is_git_repo_at_commit(source_path, &occt_version).expect("Failed to check commit ID") {
            // If the commit is different, we should fetch and checkout the commit
            fetch_origin(source_path, BRANCH).expect("Failed to fetch latest commit");
            checkout_commit(source_path, &occt_version).expect("Failed to checkout latest commit");
        }
    }
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

fn get_remote_url(source_dir: &Path) -> Result<String, String> {
    execute_git_command(&["remote", "get-url", "origin"], source_dir)
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
