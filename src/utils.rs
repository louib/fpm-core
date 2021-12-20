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
