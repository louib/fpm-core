use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub fn get_module_hash(module: &flatpak_rs::flatpak_manifest::FlatpakModuleDescription) -> String {
    // TODO maybe this should go into flatpak_rs??
    let mut s = DefaultHasher::new();
    module.hash(&mut s);
    s.finish().to_string()
}

pub fn get_all_paths(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut all_paths: Vec<std::path::PathBuf> = vec![];

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return Err(err.to_string()),
    };
    for entry in dir_entries {
        let entry_path = entry.unwrap().path();
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
///assert_eq!(formatted_bytes, "100 B");
///let formatted_bytes = fpm_core::utils::format_bytes(23746178);
///assert_eq!(formatted_bytes, "22 MB");
///let formatted_bytes = fpm_core::utils::format_bytes(9823453784599);
///assert_eq!(formatted_bytes, "8 TB");
///```
pub fn format_bytes(bytes: usize) -> String {
    let SIZES: Vec<&str> = vec!["B", "KB", "MB", "GB", "TB"];

    let mut i = 0;
    let mut approximated_bytes = bytes;
    while i < 5 && approximated_bytes >= 1024 {
        i += 1;
        approximated_bytes = approximated_bytes / 1024;
    }
    return format!("{} {}", approximated_bytes, SIZES[i]);
}
