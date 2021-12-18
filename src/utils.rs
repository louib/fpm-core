use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn get_module_hash(module: &flatpak_rs::flatpak_manifest::FlatpakModuleDescription) -> String {
    // TODO maybe this should go into flatpak_rs??
    let mut s = DefaultHasher::new();
    module.hash(&mut s);
    s.finish().to_string()
}
