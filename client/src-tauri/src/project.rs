use gate_domain::modules::project::{
    CreateProjectRequest, Project, ProjectDeleteImpact, ProjectDeleteMode, ProjectService,
    ProjectTemplate, ProjectTemplateProfile, SqliteProjectRepository, TunnelRecommendation,
    UpdateProjectRequest,
};
use serde::Serialize;
use std::{env, path::PathBuf};

use crate::utils::app_data_dir;

pub struct ProjectWorkspaceState {
    service: Option<ProjectService<SqliteProjectRepository>>,
    init_error: Option<String>,
}

impl Default for ProjectWorkspaceState {
    fn default() -> Self {
        match SqliteProjectRepository::open(project_store_path()) {
            Ok(repository) => Self {
                service: Some(ProjectService::new(repository)),
                init_error: None,
            },
            Err(error) => Self {
                service: None,
                init_error: Some(format!("PROJECT_DATABASE_INIT_FAILED:{error}")),
            },
        }
    }
}

impl ProjectWorkspaceState {
    fn service(&self) -> Result<&ProjectService<SqliteProjectRepository>, String> {
        self.service.as_ref().ok_or_else(|| {
            self.init_error
                .clone()
                .unwrap_or_else(|| "PROJECT_SERVICE_UNAVAILABLE".to_string())
        })
    }

    pub fn list(&self) -> Result<Vec<Project>, String> {
        self.service()?.list().map_err(|error| error.to_string())
    }

    pub fn get(&self, project_id: &str) -> Result<Project, String> {
        self.service()?
            .get(project_id)
            .map_err(|error| error.to_string())
    }

    pub fn create(&self, request: CreateProjectRequest) -> Result<Project, String> {
        self.service()?
            .create(request)
            .map_err(|error| error.to_string())
    }

    pub fn update(&self, project_id: &str, patch: UpdateProjectRequest) -> Result<Project, String> {
        self.service()?
            .update(project_id, patch)
            .map_err(|error| error.to_string())
    }

    pub fn delete(&self, project_id: &str, mode: ProjectDeleteMode) -> Result<Project, String> {
        self.service()?
            .delete(project_id, mode)
            .map_err(|error| error.to_string())
    }

    pub fn delete_impact(&self, project_id: &str) -> Result<ProjectDeleteImpact, String> {
        self.service()?
            .delete_impact(project_id)
            .map_err(|error| error.to_string())
    }

    pub fn set_favorite(&self, project_id: &str, favorite: bool) -> Result<Project, String> {
        self.service()?
            .set_favorite(project_id, favorite)
            .map_err(|error| error.to_string())
    }

    pub fn set_pinned(&self, project_id: &str, pinned: bool) -> Result<Project, String> {
        self.service()?
            .set_pinned(project_id, pinned)
            .map_err(|error| error.to_string())
    }

    pub fn add_tunnel(&self, project_id: &str, tunnel_id: String) -> Result<Project, String> {
        self.service()?
            .add_tunnel(project_id, tunnel_id)
            .map_err(|error| error.to_string())
    }

    pub fn remove_tunnel(&self, project_id: &str, tunnel_id: &str) -> Result<Project, String> {
        self.service()?
            .remove_tunnel(project_id, tunnel_id)
            .map_err(|error| error.to_string())
    }

    pub fn move_tunnel(
        &self,
        source_project_id: &str,
        target_project_id: &str,
        tunnel_id: &str,
    ) -> Result<(Project, Project), String> {
        self.service()?
            .move_tunnel(source_project_id, target_project_id, tunnel_id)
            .map_err(|error| error.to_string())
    }

    pub fn add_domain(&self, project_id: &str, domain: String) -> Result<Project, String> {
        self.service()?
            .add_domain(project_id, domain)
            .map_err(|error| error.to_string())
    }

    pub fn remove_domain(&self, project_id: &str, domain: &str) -> Result<Project, String> {
        self.service()?
            .remove_domain(project_id, domain)
            .map_err(|error| error.to_string())
    }

    pub fn add_certificate(
        &self,
        project_id: &str,
        certificate_id: String,
    ) -> Result<Project, String> {
        self.service()?
            .add_certificate(project_id, certificate_id)
            .map_err(|error| error.to_string())
    }

    pub fn remove_certificate(
        &self,
        project_id: &str,
        certificate_id: &str,
    ) -> Result<Project, String> {
        self.service()?
            .remove_certificate(project_id, certificate_id)
            .map_err(|error| error.to_string())
    }

    pub fn templates(&self) -> Vec<ProjectTemplateProfile> {
        self.service
            .as_ref()
            .map(|service| service.templates())
            .unwrap_or_default()
    }

    pub fn recommend_tunnels(&self, template: ProjectTemplate) -> Vec<TunnelRecommendation> {
        self.service
            .as_ref()
            .map(|service| service.recommend_tunnels(template))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDeleteResponse {
    pub project_id: String,
    pub impact: ProjectDeleteImpact,
    pub deleted_tunnel_ids: Vec<String>,
    pub failed_tunnel_ids: Vec<String>,
}

pub(crate) fn project_store_path() -> PathBuf {
    if let Some(value) = env::var_os("GATE_PROJECT_DB") {
        return PathBuf::from(value);
    }

    runtime_data_dir().join("projects.sqlite3")
}

fn runtime_data_dir() -> PathBuf {
    // 与 runtime/config/certificate 共用平台目录，保证备份和数据库位置一致。
    app_data_dir().unwrap_or_else(|| PathBuf::from(".gate"))
}
