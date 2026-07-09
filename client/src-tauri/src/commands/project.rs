use gate_domain::modules::project::{
    CreateProjectRequest, Project, ProjectDeleteImpact, ProjectDeleteMode, ProjectTemplate,
    ProjectTemplateProfile, TunnelRecommendation, UpdateProjectRequest,
};
use tauri::State;

use crate::{
    commands::error::{AppError, CommandResult},
    project::{ProjectDeleteResponse, ProjectWorkspaceState},
    runtime::ClientRuntimeState,
};

#[tauri::command]
pub async fn project_list(state: State<'_, ProjectWorkspaceState>) -> CommandResult<Vec<Project>> {
    project_result(state.list())
}

#[tauri::command]
pub async fn project_detail(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
) -> CommandResult<Project> {
    project_result(state.get(&project_id))
}

#[tauri::command]
pub async fn project_create(
    state: State<'_, ProjectWorkspaceState>,
    request: CreateProjectRequest,
) -> CommandResult<Project> {
    project_result(state.create(request))
}

#[tauri::command]
pub async fn project_update(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    patch: UpdateProjectRequest,
) -> CommandResult<Project> {
    project_result(state.update(&project_id, patch))
}

#[tauri::command]
pub async fn project_delete_impact(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
) -> CommandResult<ProjectDeleteImpact> {
    project_result(state.delete_impact(&project_id))
}

#[tauri::command]
pub async fn project_delete(
    state: State<'_, ProjectWorkspaceState>,
    runtime: State<'_, ClientRuntimeState>,
    project_id: String,
    mode: Option<ProjectDeleteMode>,
) -> CommandResult<ProjectDeleteResponse> {
    let mode = mode.unwrap_or_default();
    let project = project_result(state.get(&project_id))?;
    let impact = ProjectDeleteImpact::from_project(&project);
    let mut deleted_tunnel_ids = Vec::new();
    let mut failed_tunnel_ids = Vec::new();

    if mode == ProjectDeleteMode::CascadeResources {
        for tunnel_id in &project.tunnel_ids {
            match runtime.delete_tunnel(tunnel_id.clone()).await {
                Ok(()) => deleted_tunnel_ids.push(tunnel_id.clone()),
                Err(_) => failed_tunnel_ids.push(tunnel_id.clone()),
            }
        }
    }

    project_result(state.delete(&project_id, mode))?;

    Ok(ProjectDeleteResponse {
        project_id,
        impact,
        deleted_tunnel_ids,
        failed_tunnel_ids,
    })
}

#[tauri::command]
pub async fn project_set_favorite(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    favorite: bool,
) -> CommandResult<Project> {
    project_result(state.set_favorite(&project_id, favorite))
}

#[tauri::command]
pub async fn project_set_pinned(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    pinned: bool,
) -> CommandResult<Project> {
    project_result(state.set_pinned(&project_id, pinned))
}

#[tauri::command]
pub async fn project_add_tunnel(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    tunnel_id: String,
) -> CommandResult<Project> {
    project_result(state.add_tunnel(&project_id, tunnel_id))
}

#[tauri::command]
pub async fn project_remove_tunnel(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    tunnel_id: String,
) -> CommandResult<Project> {
    project_result(state.remove_tunnel(&project_id, &tunnel_id))
}

#[tauri::command]
pub async fn project_move_tunnel(
    state: State<'_, ProjectWorkspaceState>,
    source_project_id: String,
    target_project_id: String,
    tunnel_id: String,
) -> CommandResult<(Project, Project)> {
    project_result(state.move_tunnel(&source_project_id, &target_project_id, &tunnel_id))
}

#[tauri::command]
pub async fn project_add_domain(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    domain: String,
) -> CommandResult<Project> {
    project_result(state.add_domain(&project_id, domain))
}

#[tauri::command]
pub async fn project_remove_domain(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    domain: String,
) -> CommandResult<Project> {
    project_result(state.remove_domain(&project_id, &domain))
}

#[tauri::command]
pub async fn project_add_certificate(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    certificate_id: String,
) -> CommandResult<Project> {
    project_result(state.add_certificate(&project_id, certificate_id))
}

#[tauri::command]
pub async fn project_remove_certificate(
    state: State<'_, ProjectWorkspaceState>,
    project_id: String,
    certificate_id: String,
) -> CommandResult<Project> {
    project_result(state.remove_certificate(&project_id, &certificate_id))
}

#[tauri::command]
pub async fn project_templates(
    state: State<'_, ProjectWorkspaceState>,
) -> CommandResult<Vec<ProjectTemplateProfile>> {
    Ok(state.templates())
}

#[tauri::command]
pub async fn project_recommend_tunnels(
    state: State<'_, ProjectWorkspaceState>,
    template: ProjectTemplate,
) -> CommandResult<Vec<TunnelRecommendation>> {
    Ok(state.recommend_tunnels(template))
}

fn project_result<T>(result: Result<T, String>) -> CommandResult<T> {
    result.map_err(project_error)
}

fn project_error(source: impl std::fmt::Display) -> AppError {
    AppError::from_source(
        "PROJECT_OPERATION_FAILED",
        "errors.project.operationFailed",
        source,
    )
}
