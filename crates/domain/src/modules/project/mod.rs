pub mod entity;
pub mod error;
pub mod event;
pub mod handler;
pub mod repository;
pub mod service;
pub mod types;

pub use entity::{
    CreateProjectRequest, Project, ProjectDeleteImpact, ProjectDeleteMode, ProjectEnvironment,
    ProjectEnvironmentVariable, ProjectNote, UpdateProjectRequest,
};
pub use error::{ProjectError, ProjectResult};
pub use repository::{MemoryProjectRepository, ProjectRepository, SqliteProjectRepository};
pub use service::ProjectService;
pub use types::{
    project_template_profiles, recommendations_for_template, ProjectTemplate,
    ProjectTemplateProfile, TunnelRecommendation,
};
