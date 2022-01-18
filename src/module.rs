use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Default)]
pub struct SoftwareModule {
    pub project_id: Option<String>,

    pub flatpak_module: flatpak_rs::module::FlatpakModule,
}
