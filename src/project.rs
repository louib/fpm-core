use std::collections::HashSet;

use flatpak_rs::build_system::FlatpakBuildSystem;
use flatpak_rs::module::FlatpakModule;
use flatpak_rs::source::{FlatpakSourceItem, FlatpakSource};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Default)]
pub struct SoftwareProject {
    // Project ids are based on the reverse DNS notation, and
    // are either derived from build manifests found in the project
    // using the same reverse DNS notation, or from the urls
    // of version-control systems (VCS) repositories associated
    // with the project.
    pub id: String,

    // This is the main URL of the project, and also the one
    // that was used to generate the project id.
    pub vcs_url: String,

    // Common name of the software project.
    pub name: String,

    // Description of the software project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub web_urls: HashSet<String>,

    pub vcs_urls: HashSet<String>,

    // A set of all the known siblings. Sibling have the same
    // root hashes fingerprint, but could be forks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub siblings: Option<HashSet<String>>,

    // A list of the paths of known flatpak app manifests found
    // in the project's repository.
    pub flatpak_app_manifests: HashSet<String>,

    // A list of the paths of known flatpak module definition manifests found
    // in the project's repository.
    pub flatpak_module_manifests: HashSet<String>,

    // A list of the paths of known flatpak sources definition manifests found
    // in the project's repository.
    pub flatpak_sources_manifests: HashSet<String>,

    // A list of tags associated with the project.
    // Those include the sources from which a project was discovered.
    pub tags: HashSet<String>,

    // All the build systems that are known to be supported by the project.
    pub build_systems: HashSet<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_branch: Option<String>,

    // Hash of the latest commit on the main branch seen during the
    // latest update of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_known_commit: Option<String>,

    // When the project's main branch was last updated
    // in the local git checkout of the project.
    // Stored as an ISO datetime string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,

    // The root git commit hashes associated with the project. This is used
    // for project de-duplication, in the case a project has multiple remote
    // git repositories. I used a vector instead of a set, because
    // I believe it's possible to have two ancestors with the same hash.
    pub root_hashes: Vec<String>,
}
impl SoftwareProject {
    pub fn merge(&mut self, other_project: &SoftwareProject) {
        if self.id != other_project.id {
            panic!(
                "Cannot merge projects with different IDs! {} != {}",
                self.id, other_project.id
            );
        }
        if self.vcs_url != other_project.vcs_url {
            panic!(
                "Cannot merge projects with different VCS URLs! {} != {}",
                self.vcs_url, other_project.vcs_url
            );
        }
        for web_url in &other_project.web_urls {
            self.web_urls.insert(web_url.to_string());
        }
        for vcs_url in &other_project.vcs_urls {
            self.vcs_urls.insert(vcs_url.to_string());
        }
        for build_system in &other_project.build_systems {
            self.build_systems.insert(build_system.to_string());
        }
        for app_manifest in &other_project.flatpak_app_manifests {
            self.flatpak_app_manifests.insert(app_manifest.to_string());
        }
        for module_manifest in &other_project.flatpak_module_manifests {
            self.flatpak_module_manifests.insert(module_manifest.to_string());
        }
        for source_manifest in &other_project.flatpak_sources_manifests {
            self.flatpak_sources_manifests.insert(source_manifest.to_string());
        }
        for tag in &other_project.tags {
            self.tags.insert(tag.to_string());
        }
        if self.root_hashes.len() == 0 {
            for hash in &other_project.root_hashes {
                self.root_hashes.push(hash.clone());
            }
        }
        if let Some(siblings) = &other_project.siblings {
            if self.siblings.is_none() {
                self.siblings = other_project.siblings.clone();
            }
        }
        if let Some(description) = &other_project.description {
            self.description = Some(description.clone());
        }
        if let Some(last_known_commit) = &other_project.last_known_commit {
            self.last_known_commit = Some(last_known_commit.clone());
        }
        if let Some(main_branch) = &other_project.main_branch {
            self.main_branch = Some(main_branch.clone());
        }
        // TODO maybe we should preserve the most recent one instead.
        if let Some(last_updated) = &other_project.last_updated {
            self.last_updated = Some(last_updated.clone());
        }
    }

    pub fn supports_flatpak(&self) -> bool {
        if !self.flatpak_app_manifests.is_empty() {
            return true;
        }
        if !self.flatpak_module_manifests.is_empty() {
            return true;
        }
        if !self.flatpak_sources_manifests.is_empty() {
            return true;
        }
        return false;
    }

    pub fn get_main_vcs_url(&self) -> String {
        return self.vcs_url.to_string();
    }

    pub fn get_root_signature(&self) -> String {
        let mut root_signature: String = "".to_string();
        for root_hash in &self.root_hashes {
            root_signature += root_hash;
        }
        root_signature
    }

    pub fn get_default_modules(&self) -> Vec<FlatpakModule> {
        let mut response = vec![];
        let mut git_source = FlatpakSource::default();
        git_source.url = Some(self.vcs_url.clone());
        if let Some(branch) = &self.main_branch {
            git_source.branch = Some(branch.clone());
        }
        // TODO also set the mirror urls if available.

        for build_system in &self.build_systems {
            let buildsystem = match FlatpakBuildSystem::from_string(build_system) {
                Err(e) => {
                    // This is not a build system supported by Flatpak. It's not a big deal.
                    continue;
                }
                Ok(b) => b,
            };
            let mut derived_module = FlatpakModule::default();
            derived_module.name = self.id.clone();
            derived_module.buildsystem = buildsystem;
            derived_module
                .sources
                .push(FlatpakSourceItem::Description(git_source.clone()));
            response.push(derived_module);
        }
        response
    }
}
