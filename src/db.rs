use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::mem;
use std::path;
use std::time::Instant;

use crate::module::SoftwareModule;
use crate::project::SoftwareProject;
use flatpak_rs::module::FlatpakModule;

pub const PROJECTS_DB_SUBDIR: &str = "/projects";
pub const MODULES_DB_SUBDIR: &str = "/modules";
pub const MANIFESTS_DB_SUBDIR: &str = "/manifests";

pub struct Database {
    pub indexed_projects: BTreeMap<String, SoftwareProject>,
    pub modules: Vec<SoftwareModule>,
}
impl Database {
    pub fn get_database() -> Database {
        if let Err(e) = fs::create_dir_all(Database::get_projects_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }
        if let Err(e) = fs::create_dir_all(Database::get_manifests_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }
        if let Err(e) = fs::create_dir_all(Database::get_modules_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }

        let before_loading = Instant::now();
        let database = Database {
            modules: Database::get_all_modules(),
            indexed_projects: Database::get_indexed_projects(),
        };
        let loading_duration = before_loading.elapsed();
        if loading_duration.as_secs() == 0 {
            log::info!("Loading the database took {}ms.", loading_duration.as_millis());
        } else {
            log::info!("Loading the database took {}s.", loading_duration.as_secs());
        }

        database
    }

    pub fn get_indexed_projects() -> BTreeMap<String, SoftwareProject> {
        let mut indexed_projects: BTreeMap<String, SoftwareProject> = BTreeMap::new();
        for project in Database::get_all_projects() {
            indexed_projects.insert(project.id.clone(), project);
        }
        indexed_projects
    }

    pub fn get_stats(&self) -> String {
        let mut response = "".to_string();
        response += &format!("Modules: {}.\n", self.modules.len());

        let mut updateable_module_count = 0;

        for module in &self.modules {
            if module.flatpak_module.uses_external_data_checker() {
                updateable_module_count += 1;
            }
        }
        response += &format!("Modules supporting updates: {}.\n", updateable_module_count);

        response += &format!("Projects: {}.\n", self.indexed_projects.len());
        response += &format!(
            "Database in-memory size: {}.\n",
            crate::utils::format_bytes(self.get_database_memory_size())
        );

        let mut root_signatures: HashSet<String> = HashSet::new();
        let mut unmined_projects = 0;
        let mut inaccessible_projects = 0;
        let mut projects_with_siblings = 0;
        let mut projects_with_build_systems: i64 = 0;
        let mut build_systems_count: BTreeMap<String, i64> = BTreeMap::new();
        for (_project_id, project) in &self.indexed_projects {
            if project.root_hashes.len() == 0 {
                if project.last_updated.is_some() {
                    inaccessible_projects += 1;
                } else {
                    unmined_projects += 1;
                }
            }
            if project.build_systems.len() != 0 {
                projects_with_build_systems += 1;
            }

            for build_system in &project.build_systems {
                let new_build_system_count = build_systems_count.get(build_system).unwrap_or(&0) + 1;
                build_systems_count.insert(build_system.to_string(), new_build_system_count);
            }

            let root_signature = project.get_root_signature();
            if root_signature.len() == 0 {
                continue;
            }
            root_signatures.insert(root_signature);

            if let Some(siblings) = &project.siblings {
                if siblings.len() != 0 {
                    projects_with_siblings += 1;
                }
            }
        }

        response += &format!(
            "{:.2}% ({}/{}) of the projects are unmined.\n",
            (unmined_projects as f64 / self.indexed_projects.len() as f64) * 100.0,
            unmined_projects,
            self.indexed_projects.len(),
        );
        response += &format!(
            "{:.2}% ({}/{}) of the projects are inaccessible.\n",
            (inaccessible_projects as f64 / self.indexed_projects.len() as f64) * 100.0,
            inaccessible_projects,
            self.indexed_projects.len(),
        );
        response += &format!(
            "{:05.2}% of the projects have a build system.\n",
            (projects_with_build_systems as f64 / self.indexed_projects.len() as f64) * 100.0,
        );

        for (build_system, build_system_count) in build_systems_count {
            response += &format!(
                "{:05.2}% Projects use {}\n",
                (build_system_count as f64 / self.indexed_projects.len() as f64) * 100.0,
                build_system,
            );
        }

        response += &format!("{} Unique root signatures.\n", root_signatures.len());
        response += &format!(
            "{: <25}: {:>5.2}% ({}/{})\n",
            "Projects with siblings",
            (projects_with_siblings as f64 / self.indexed_projects.len() as f64) * 100.0,
            projects_with_siblings,
            self.indexed_projects.len(),
        );

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
        for (project_id, project) in &self.indexed_projects {
            db_size += mem::size_of_val(project_id);
            db_size += mem::size_of_val(project);
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

    pub fn get_projects_db_path() -> String {
        Database::get_db_path() + PROJECTS_DB_SUBDIR
    }

    pub fn get_manifests_db_path() -> String {
        Database::get_db_path() + MANIFESTS_DB_SUBDIR
    }

    pub fn get_modules_db_path() -> String {
        Database::get_db_path() + MODULES_DB_SUBDIR
    }

    pub fn get_all_projects() -> Vec<SoftwareProject> {
        let projects_path = Database::get_projects_db_path();
        let projects_path = path::Path::new(&projects_path);
        let all_projects_paths = match crate::utils::get_all_paths(projects_path) {
            Ok(paths) => paths,
            Err(e) => {
                log::error!("Could not get projects' paths: {}", e);
                return vec![];
            }
        };
        let mut projects: Vec<SoftwareProject> = vec![];
        for project_path in all_projects_paths.iter() {
            let project_path_str = project_path.to_str().unwrap();
            if !project_path.is_file() {
                log::debug!("{} is not a file.", &project_path_str);
                continue;
            }
            // Don't even try to open it if it's not a yaml file.
            if !project_path_str.ends_with("yml") && !project_path_str.ends_with("yaml") {
                continue;
            }
            let project_content = match fs::read_to_string(project_path) {
                Ok(content) => content,
                Err(e) => panic!("Could not read project file {}: {}.", &project_path_str, e),
            };
            let project = match serde_yaml::from_str(&project_content) {
                Ok(p) => p,
                Err(e) => panic!("Could not parse project file at {}: {}.", &project_path_str, e),
            };
            projects.push(project);
        }
        projects
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

    pub fn update_project(&mut self, project: &SoftwareProject) {
        let projects_path = Database::get_projects_db_path();
        if project.id.len() == 0 {
            panic!("Trying to update a project to the db without an id!");
        }
        let existing_project = self.indexed_projects.get_mut(&project.id).unwrap();

        let new_project_path = format!("{}/{}.yaml", projects_path, &project.id);
        let project_fs_path = path::Path::new(&new_project_path);
        if !project_fs_path.exists() {
            panic!("Project {} does not exist", project.id);
        }
        log::info!("Updating project at {}", new_project_path);

        existing_project.merge(project);
        match fs::write(project_fs_path, serde_yaml::to_string(&existing_project).unwrap()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!(
                    "Could not write new project at {}: {}",
                    new_project_path.to_string(),
                    e
                );
            }
        };
    }

    pub fn add_project(&mut self, project: SoftwareProject) {
        let projects_path = Database::get_projects_db_path();
        if project.id.len() == 0 {
            panic!("Trying to add a project to the db without an id!");
        }
        let project_path = format!("{}/{}.yaml", projects_path, &project.id);
        log::info!("Adding project at {}", project_path);
        let new_project_fs_path = path::Path::new(&project_path);
        if new_project_fs_path.exists() {
            return self.update_project(&project);
        }
        match fs::write(new_project_fs_path, serde_yaml::to_string(&project).unwrap()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!(
                    "Could not write new project at {}: {}",
                    project_path.to_string(),
                    e
                );
            }
        };
        self.indexed_projects.insert(project.id.clone(), project);
    }

    pub fn search_projects(&self, search_term: &str) -> Vec<&SoftwareProject> {
        let mut projects: Vec<&SoftwareProject> = vec![];
        for (_, project) in &self.indexed_projects {
            if project.name.contains(&search_term) {
                projects.push(&project);
            }
            if project.vcs_url.contains(&search_term) {
                projects.push(&project);
            }
        }
        projects
    }

    pub fn get_project(&self, project_id: &str) -> Option<SoftwareProject> {
        return match self.indexed_projects.get(project_id) {
            Some(p) => Some((*p).clone()),
            None => None,
        };
    }

    pub fn has_project(&self, project_id: &str) -> bool {
        self.indexed_projects.contains_key(project_id)
    }

    pub fn detect_siblings(&mut self) {
        // A map of all the known root hash signatures (merged into a single string), mapped to all
        // the known siblings.
        let mut siblings_for_root_signature: BTreeMap<String, HashSet<String>> = BTreeMap::new();
        for (project_id, project) in &self.indexed_projects {
            let root_signature = project.get_root_signature();
            if root_signature.len() == 0 {
                continue;
            }

            if let Some(siblings) = siblings_for_root_signature.get(&root_signature) {
                let mut new_siblings = siblings.clone();
                new_siblings.insert(project_id.clone());
                siblings_for_root_signature.insert(root_signature, new_siblings);
            } else {
                let mut siblings: HashSet<String> = HashSet::new();
                siblings.insert(project_id.clone());
                siblings_for_root_signature.insert(root_signature, siblings);
            }
        }

        for siblings in siblings_for_root_signature.values() {
            // No real need to store it when there's only one (self).
            if siblings.len() <= 1 {
                continue;
            }
            for sibling in siblings.iter() {
                let mut project = self.get_project(sibling).unwrap().clone();
                project.siblings = Some(siblings.clone());
                self.update_project(&project);
            }
        }
    }
}
