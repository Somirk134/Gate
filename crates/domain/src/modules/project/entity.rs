use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::ProjectTemplate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEnvironmentVariable {
    pub key: String,
    pub value: String,
    pub secret: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEnvironment {
    pub id: String,
    pub name: String,
    pub variables: Vec<ProjectEnvironmentVariable>,
}

impl ProjectEnvironment {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            variables: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ProjectNote {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = now_ms();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            content: content.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub template: ProjectTemplate,
    pub tunnel_ids: Vec<String>,
    pub domains: Vec<String>,
    pub certificate_ids: Vec<String>,
    pub tags: Vec<String>,
    pub environments: Vec<ProjectEnvironment>,
    pub notes: Vec<ProjectNote>,
    pub favorite: bool,
    pub pinned: bool,
    pub auto_start: bool,
    pub startup_policy: Option<String>,
    pub last_activity_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Project {
    pub fn new(request: CreateProjectRequest) -> Self {
        let now = now_ms();
        let template = request.template.unwrap_or_default();

        Self {
            id: Uuid::new_v4().to_string(),
            name: normalize_name(&request.name),
            description: request.description.unwrap_or_default(),
            icon: request.icon.unwrap_or_else(|| "package".to_string()),
            color: request.color.unwrap_or_else(|| "blue".to_string()),
            template,
            tunnel_ids: unique_non_empty(request.tunnel_ids.unwrap_or_default()),
            domains: unique_non_empty(request.domains.unwrap_or_default()),
            certificate_ids: unique_non_empty(request.certificate_ids.unwrap_or_default()),
            tags: unique_non_empty(request.tags.unwrap_or_default()),
            environments: request.environments.unwrap_or_default(),
            notes: request.notes.unwrap_or_default(),
            favorite: request.favorite.unwrap_or(false),
            pinned: request.pinned.unwrap_or(false),
            auto_start: request.auto_start.unwrap_or(false),
            startup_policy: request
                .startup_policy
                .flatten()
                .filter(|value| !value.trim().is_empty()),
            last_activity_at: now,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_patch(&mut self, patch: UpdateProjectRequest) {
        if let Some(name) = patch.name {
            self.name = normalize_name(&name);
        }
        if let Some(description) = patch.description {
            self.description = description;
        }
        if let Some(icon) = patch.icon {
            self.icon = normalize_optional_text(icon, "package");
        }
        if let Some(color) = patch.color {
            self.color = normalize_optional_text(color, "blue");
        }
        if let Some(template) = patch.template {
            self.template = template;
        }
        if let Some(tunnel_ids) = patch.tunnel_ids {
            self.tunnel_ids = unique_non_empty(tunnel_ids);
        }
        if let Some(domains) = patch.domains {
            self.domains = unique_non_empty(domains);
        }
        if let Some(certificate_ids) = patch.certificate_ids {
            self.certificate_ids = unique_non_empty(certificate_ids);
        }
        if let Some(tags) = patch.tags {
            self.tags = unique_non_empty(tags);
        }
        if let Some(environments) = patch.environments {
            self.environments = environments;
        }
        if let Some(notes) = patch.notes {
            self.notes = notes;
        }
        if let Some(favorite) = patch.favorite {
            self.favorite = favorite;
        }
        if let Some(pinned) = patch.pinned {
            self.pinned = pinned;
        }
        if let Some(auto_start) = patch.auto_start {
            self.auto_start = auto_start;
        }
        if let Some(startup_policy) = patch.startup_policy {
            self.startup_policy = startup_policy.filter(|value| !value.trim().is_empty());
        }
        self.touch();
    }

    pub fn touch(&mut self) {
        let now = now_ms();
        self.updated_at = now;
        self.last_activity_at = now;
    }

    pub fn add_tunnel(&mut self, tunnel_id: impl Into<String>) {
        push_unique(&mut self.tunnel_ids, tunnel_id);
        self.touch();
    }

    pub fn remove_tunnel(&mut self, tunnel_id: &str) {
        self.tunnel_ids.retain(|id| id != tunnel_id);
        self.touch();
    }

    pub fn add_domain(&mut self, domain: impl Into<String>) {
        push_unique(&mut self.domains, domain);
        self.touch();
    }

    pub fn remove_domain(&mut self, domain: &str) {
        self.domains.retain(|item| item != domain);
        self.touch();
    }

    pub fn add_certificate(&mut self, certificate_id: impl Into<String>) {
        push_unique(&mut self.certificate_ids, certificate_id);
        self.touch();
    }

    pub fn remove_certificate(&mut self, certificate_id: &str) {
        self.certificate_ids.retain(|item| item != certificate_id);
        self.touch();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub template: Option<ProjectTemplate>,
    pub tunnel_ids: Option<Vec<String>>,
    pub domains: Option<Vec<String>>,
    pub certificate_ids: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub environments: Option<Vec<ProjectEnvironment>>,
    pub notes: Option<Vec<ProjectNote>>,
    pub favorite: Option<bool>,
    pub pinned: Option<bool>,
    pub auto_start: Option<bool>,
    pub startup_policy: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub template: Option<ProjectTemplate>,
    pub tunnel_ids: Option<Vec<String>>,
    pub domains: Option<Vec<String>>,
    pub certificate_ids: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub environments: Option<Vec<ProjectEnvironment>>,
    pub notes: Option<Vec<ProjectNote>>,
    pub favorite: Option<bool>,
    pub pinned: Option<bool>,
    pub auto_start: Option<bool>,
    pub startup_policy: Option<Option<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectDeleteMode {
    ProjectOnly,
    CascadeResources,
}

impl Default for ProjectDeleteMode {
    fn default() -> Self {
        Self::ProjectOnly
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDeleteImpact {
    pub tunnel_count: usize,
    pub domain_count: usize,
    pub certificate_count: usize,
    pub has_references: bool,
}

impl ProjectDeleteImpact {
    pub fn from_project(project: &Project) -> Self {
        let tunnel_count = project.tunnel_ids.len();
        let domain_count = project.domains.len();
        let certificate_count = project.certificate_ids.len();

        Self {
            tunnel_count,
            domain_count,
            certificate_count,
            has_references: tunnel_count > 0 || domain_count > 0 || certificate_count > 0,
        }
    }
}

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn normalize_name(value: &str) -> String {
    value.trim().to_string()
}

pub fn unique_non_empty(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !normalized.iter().any(|item| item == trimmed) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn push_unique(values: &mut Vec<String>, value: impl Into<String>) {
    let normalized = normalize_name(&value.into());
    if !normalized.is_empty() && !values.iter().any(|item| item == &normalized) {
        values.push(normalized);
    }
}

fn normalize_optional_text(value: String, fallback: &str) -> String {
    let normalized = normalize_name(&value);
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized
    }
}
