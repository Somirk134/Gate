use super::{
    entity::{
        CreateProjectRequest, Project, ProjectDeleteImpact, ProjectDeleteMode, UpdateProjectRequest,
    },
    error::{ProjectError, ProjectResult},
    repository::ProjectRepository,
    types::{
        project_template_profiles, recommendations_for_template, ProjectTemplate,
        ProjectTemplateProfile, TunnelRecommendation,
    },
};

#[derive(Clone)]
pub struct ProjectService<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> ProjectService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn list(&self) -> ProjectResult<Vec<Project>> {
        self.repository.list()
    }

    pub fn find_by_id(&self, id: &str) -> ProjectResult<Option<Project>> {
        self.repository.find_by_id(id)
    }

    pub fn get(&self, id: &str) -> ProjectResult<Project> {
        self.repository
            .find_by_id(id)?
            .ok_or_else(|| ProjectError::NotFound(id.to_string()))
    }

    pub fn create(&self, request: CreateProjectRequest) -> ProjectResult<Project> {
        validate_name(&request.name)?;
        let project = Project::new(request);
        self.repository.create(project)
    }

    pub fn update(&self, id: &str, patch: UpdateProjectRequest) -> ProjectResult<Project> {
        if let Some(name) = &patch.name {
            validate_name(name)?;
        }
        self.modify(id, |project| project.apply_patch(patch))
    }

    pub fn delete(&self, id: &str, _mode: ProjectDeleteMode) -> ProjectResult<Project> {
        self.repository
            .delete(id)?
            .ok_or_else(|| ProjectError::NotFound(id.to_string()))
    }

    pub fn delete_impact(&self, id: &str) -> ProjectResult<ProjectDeleteImpact> {
        let project = self.get(id)?;
        Ok(ProjectDeleteImpact::from_project(&project))
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> ProjectResult<Project> {
        self.modify(id, |project| {
            project.favorite = favorite;
            project.touch();
        })
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> ProjectResult<Project> {
        self.modify(id, |project| {
            project.pinned = pinned;
            project.touch();
        })
    }

    pub fn add_tunnel(&self, id: &str, tunnel_id: impl Into<String>) -> ProjectResult<Project> {
        let tunnel_id = tunnel_id.into();
        self.modify(id, |project| project.add_tunnel(tunnel_id))
    }

    pub fn remove_tunnel(&self, id: &str, tunnel_id: &str) -> ProjectResult<Project> {
        self.modify(id, |project| project.remove_tunnel(tunnel_id))
    }

    pub fn move_tunnel(
        &self,
        source_project_id: &str,
        target_project_id: &str,
        tunnel_id: &str,
    ) -> ProjectResult<(Project, Project)> {
        let mut source = self.get(source_project_id)?;
        let mut target = self.get(target_project_id)?;

        source.remove_tunnel(tunnel_id);
        target.add_tunnel(tunnel_id.to_string());

        let source = self.repository.update(source)?;
        let target = self.repository.update(target)?;
        Ok((source, target))
    }

    pub fn add_domain(&self, id: &str, domain: impl Into<String>) -> ProjectResult<Project> {
        let domain = domain.into();
        self.modify(id, |project| project.add_domain(domain))
    }

    pub fn remove_domain(&self, id: &str, domain: &str) -> ProjectResult<Project> {
        self.modify(id, |project| project.remove_domain(domain))
    }

    pub fn add_certificate(
        &self,
        id: &str,
        certificate_id: impl Into<String>,
    ) -> ProjectResult<Project> {
        let certificate_id = certificate_id.into();
        self.modify(id, |project| project.add_certificate(certificate_id))
    }

    pub fn remove_certificate(&self, id: &str, certificate_id: &str) -> ProjectResult<Project> {
        self.modify(id, |project| project.remove_certificate(certificate_id))
    }

    pub fn templates(&self) -> Vec<ProjectTemplateProfile> {
        project_template_profiles()
    }

    pub fn recommend_tunnels(&self, template: ProjectTemplate) -> Vec<TunnelRecommendation> {
        recommendations_for_template(template)
    }

    fn modify<F>(&self, id: &str, mutate: F) -> ProjectResult<Project>
    where
        F: FnOnce(&mut Project),
    {
        let mut project = self.get(id)?;
        mutate(&mut project);
        self.repository.update(project)
    }
}

fn validate_name(value: &str) -> ProjectResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProjectError::NameRequired);
    }
    if trimmed.chars().count() > 80 {
        return Err(ProjectError::InvalidField(
            "project name cannot exceed 80 characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::project::repository::MemoryProjectRepository;

    #[test]
    fn service_creates_and_updates_project_references() -> anyhow::Result<()> {
        let service = ProjectService::new(MemoryProjectRepository::new());
        let project = service.create(CreateProjectRequest {
            name: "Workspace".to_string(),
            ..Default::default()
        })?;

        let project = service.add_tunnel(&project.id, "tunnel-1")?;
        let project = service.add_domain(&project.id, "example.test")?;
        let project = service.add_certificate(&project.id, "cert-example")?;

        assert_eq!(project.tunnel_ids, vec!["tunnel-1"]);
        assert_eq!(project.domains, vec!["example.test"]);
        assert_eq!(project.certificate_ids, vec!["cert-example"]);
        assert!(service.delete_impact(&project.id)?.has_references);
        Ok(())
    }
}
