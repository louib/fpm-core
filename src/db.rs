use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::mem;
use std::path;
use std::time::Instant;

use crate::module::SoftwareModule;
use flatpak_rs::module::FlatpakModule;

pub const MODULES_DB_SUBDIR: &str = "/modules";

pub struct Database {
    pub modules: Vec<SoftwareModule>,
}
impl Database {
    pub fn get_database() -> Database {
        if let Err(e) = fs::create_dir_all(Database::get_modules_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }

        let before_loading = Instant::now();
        let database = Database {
            modules: Database::get_all_modules(),
        };
        let loading_duration = before_loading.elapsed();
        if loading_duration.as_secs() == 0 {
            log::info!("Loading the database took {}ms.", loading_duration.as_millis());
        } else {
            log::info!("Loading the database took {}s.", loading_duration.as_secs());
        }

        database
    }

    pub fn get_stats(&self) -> String {
        let mut response = "".to_string();
        response += &format!("Modules: {}.\n", self.modules.len());

        let mut updateable_module_count = 0;
        let mut module_build_systems_count: BTreeMap<String, i64> = BTreeMap::new();

        for module in &self.modules {
            if module.flatpak_module.uses_external_data_checker() {
                updateable_module_count += 1;
            }
            if let Some(build_system) = &module.flatpak_module.buildsystem {
                let build_system_name = build_system.to_string();
                let new_build_system_count =
                    module_build_systems_count.get(&build_system_name).unwrap_or(&0) + 1;
                module_build_systems_count.insert(build_system_name.to_string(), new_build_system_count);
            }
        }
        response += &format!("Modules supporting updates: {}.\n", updateable_module_count);

        response += &format!(
            "Database in-memory size: {}.\n",
            crate::utils::format_bytes(self.get_database_memory_size())
        );

        for (build_system, build_system_count) in module_build_systems_count {
            response += &format!(
                "{:05.2}% Modules use {}\n",
                (build_system_count as f64 / self.modules.len() as f64) * 100.0,
                build_system,
            );
        }

        // TODO print module type stats.
        // TODO print archive type stats.
        // TODO print domain (URL domain) stats.
        // TODO print domain (URL domain) stats for the main vcs repo.
        // TODO add the number of archive urls.

        response
    }

    pub fn get_database_memory_size(&self) -> usize {
        let mut db_size = 0;
        for module in &self.modules {
            db_size += mem::size_of_val(module);
        }
        return db_size;
    }

    pub fn get_db_path() -> String {
        let default_db_path: String = match env::var("HOME") {
            Ok(h) => format!("{}/.fpm-db", h),
            Err(_e) => ".fpm-db".to_string(),
        };

        let db_path = match env::var("FPM_DB_DIR") {
            Ok(p) => p,
            Err(_e) => {
                log::debug!("FPM_DB_DIR is not defined. Defaulting to {}.", default_db_path);
                return default_db_path;
            }
        };
        if let Err(e) = fs::create_dir_all(&db_path) {
            panic!("Could not initialize DB directory: {}.", e);
        }
        db_path
    }

    pub fn get_modules_db_path() -> String {
        Database::get_db_path() + MODULES_DB_SUBDIR
    }

    pub fn get_all_modules() -> Vec<SoftwareModule> {
        let modules_path = Database::get_modules_db_path();
        let modules_path = path::Path::new(&modules_path);
        let all_modules_paths = match crate::utils::get_all_paths(modules_path) {
            Ok(paths) => paths,
            Err(e) => {
                log::error!("Could not get modules from database: {}.", e);
                return vec![];
            }
        };
        let mut modules: Vec<SoftwareModule> = vec![];
        for module_path in all_modules_paths.iter() {
            let module_path_str = module_path.to_str().unwrap();
            if !module_path.is_file() {
                log::debug!("{} is not a file.", &module_path_str);
                continue;
            }
            // Don't even try to open it if it's not a yaml file.
            if !module_path_str.ends_with("yml") && !module_path_str.ends_with("yaml") {
                continue;
            }
            let module_content = match fs::read_to_string(module_path) {
                Ok(content) => content,
                Err(e) => {
                    log::debug!("Could not read module file {}: {}.", &module_path_str, e);
                    continue;
                }
            };
            let module = match serde_yaml::from_str(&module_content) {
                Ok(m) => m,
                Err(e) => {
                    log::debug!("Could not parse module file at {}: {}.", &module_path_str, e);
                    continue;
                }
            };
            modules.push(module);
        }
        modules
    }

    pub fn search_modules(&self, search_term: &str) -> Vec<&FlatpakModule> {
        let mut modules: Vec<&FlatpakModule> = vec![];
        for module in &self.modules {
            if module
                .flatpak_module
                .name
                .to_lowercase()
                .contains(&search_term.to_lowercase())
            {
                modules.push(&module.flatpak_module);
            }
        }
        modules
    }

    pub fn remove_module() {}

    pub fn add_module(&mut self, new_module: FlatpakModule) {
        let module_hash = crate::utils::get_module_hash(&new_module);
        let mut new_software_module = SoftwareModule::default();
        new_software_module.flatpak_module = new_module;

        let modules_path = Database::get_modules_db_path();
        let new_module_path = format!("{}/{}.yaml", modules_path, module_hash);
        log::info!("Adding module at {}", new_module_path);
        let new_module_fs_path = path::Path::new(&new_module_path);
        if new_module_fs_path.exists() {
            // The path is based on a hash of the module, so there should be no need to
            // update a file that exists.
            return;
        }
        match fs::write(
            new_module_fs_path,
            serde_yaml::to_string(&new_software_module).unwrap(),
        ) {
            Ok(content) => content,
            Err(e) => {
                eprintln!(
                    "Could not write new module at {}: {}",
                    new_module_path.to_string(),
                    e
                );
            }
        };
        self.modules.push(new_software_module);
    }
}
