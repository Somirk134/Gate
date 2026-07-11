pub mod entity;
pub mod error;
pub mod repository;
pub mod service;
pub mod types;

pub use entity::{
    CreateProjectRequest, Project, ProjectDeleteImpact, ProjectDeleteMode, ProjectEnvironment,
    ProjectEnvironmentVariable, ProjectNote, UpdateProjectRequest,
};
pub use error::{ProjectError, ProjectResult};
#[cfg(test)]
pub use repository::MemoryProjectRepository;
pub use repository::{ProjectRepository, SqliteProjectRepository};
pub use service::ProjectService;
pub use types::{
    project_template_profiles, recommendations_for_template, ProjectTemplate,
    ProjectTemplateProfile, TunnelRecommendation,
};
