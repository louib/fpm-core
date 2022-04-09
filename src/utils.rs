use std::io::{stdin, stdout, Write};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub const DEFAULT_FLATPAK_BUILDER_CACHE_DIR: &str = ".flatpak-builder/";
pub const DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR: &str = ".flatpak-builder-out/";
pub const DEFAULT_GIT_CACHE_DIR: &str = ".git/";

pub fn get_module_hash(module: &flatpak_rs::module::FlatpakModule) -> String {
    // TODO maybe this should go into flatpak_rs??
    let mut s = DefaultHasher::new();
    module.hash(&mut s);
    s.finish().to_string()
}

pub fn get_all_paths(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut all_paths: Vec<std::path::PathBuf> = vec![];

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            return Err(format!(
                "Could not read dir {}: {}",
                dir.to_str().unwrap(),
                err.to_string()
            ))
        }
    };
    for entry in dir_entries {
        let entry_path = entry.unwrap().path();
        let entry_path_str = match entry_path.to_str() {
            Some(p) => p,
            None => continue,
        };
        if entry_path_str.contains(DEFAULT_GIT_CACHE_DIR) {
            continue;
        }
        if entry_path_str.contains(DEFAULT_FLATPAK_BUILDER_CACHE_DIR) {
            continue;
        }
        if entry_path_str.contains(DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR) {
            continue;
        }

        if entry_path.is_dir() {
            let mut dir_paths: Vec<std::path::PathBuf> = get_all_paths(&entry_path)?;
            all_paths.append(&mut dir_paths);
        } else {
            all_paths.push(entry_path);
        }
    }

    Ok(all_paths)
}

pub fn get_build_system(file_path: String) -> Option<String> {
    if file_path.ends_with("CMakeLists.txt") {
        return Some("cmake".to_string());
    }
    if file_path.ends_with("autogen.sh") || file_path.ends_with("autogen") {
        return Some("autotools".to_string());
    }
    if file_path.ends_with("bootstrap.sh") || file_path.ends_with("bootstrap") {
        return Some("autotools".to_string());
    }
    if file_path.ends_with(".pro") {
        return Some("qmake".to_string());
    }
    if file_path.ends_with("meson.build") || file_path.ends_with("meson_options.txt") {
        return Some("meson".to_string());
    }
    if file_path.ends_with("Cargo.toml") || file_path.ends_with("Cargo.lock") {
        return Some("cargo".to_string());
    }
    if file_path.ends_with("pom.xml") {
        return Some("maven".to_string());
    }
    if file_path.ends_with("debian/control") {
        return Some("debian".to_string());
    }
    if file_path.ends_with("snapcraft.yml") || file_path.ends_with("snapcraft.yaml") {
        return Some("snap".to_string());
    }
    if file_path.ends_with("go.mod") || file_path.ends_with("go.sum") {
        return Some("go".to_string());
    }
    if file_path.ends_with("package.json") || file_path.ends_with("package-lock.json") {
        return Some("npm".to_string());
    }
    if file_path.ends_with("pyproject.toml") {
        return Some("pip".to_string());
    }
    if file_path.ends_with("vcpkg.json") {
        return Some("vcpkg".to_string());
    }

    None
}

///```
///let formatted_bytes = fpm_core::utils::format_bytes(100);
///assert_eq!(formatted_bytes, "100.00 B");
///let formatted_bytes = fpm_core::utils::format_bytes(23746178);
///assert_eq!(formatted_bytes, "22.65 MB");
///let formatted_bytes = fpm_core::utils::format_bytes(9823453784599);
///assert_eq!(formatted_bytes, "8.93 TB");
///let formatted_bytes = fpm_core::utils::format_bytes(7124362542);
///assert_eq!(formatted_bytes, "6.64 GB");
///let formatted_bytes = fpm_core::utils::format_bytes(0);
///assert_eq!(formatted_bytes, "0.00 B");
///```
pub fn format_bytes(bytes: usize) -> String {
    let sizes: Vec<&str> = vec!["B", "KB", "MB", "GB", "TB"];

    let mut i = 0;
    let mut approximated_bytes = bytes as f64;
    while i < 5 && approximated_bytes >= 1024.0 {
        i += 1;
        approximated_bytes = approximated_bytes / 1024.0;
    }
    return format!("{:.2} {}", approximated_bytes, sizes[i]);
}

pub fn ask_yes_no_question(question: String) -> bool {
    let mut answer = String::new();
    print!("{}? [Y/n]: ", question);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut answer)
        .expect("Error while reading answer for question.");
    if let Some('\n') = answer.chars().next_back() {
        answer.pop();
    }
    if let Some('\r') = answer.chars().next_back() {
        answer.pop();
    }
    if answer == "Y" || answer == "y" {
        return true;
    }
    return false;
}
