use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    entity::{Project, ProjectEnvironment, ProjectNote},
    error::ProjectResult,
    types::ProjectTemplate,
};

pub trait ProjectRepository: Send + Sync {
    fn create(&self, project: Project) -> ProjectResult<Project>;
    fn update(&self, project: Project) -> ProjectResult<Project>;
    fn delete(&self, id: &str) -> ProjectResult<Option<Project>>;
    fn find_by_id(&self, id: &str) -> ProjectResult<Option<Project>>;
    fn list(&self) -> ProjectResult<Vec<Project>>;
}

#[derive(Clone, Default)]
pub struct MemoryProjectRepository {
    projects: Arc<RwLock<BTreeMap<String, Project>>>,
}

impl MemoryProjectRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProjectRepository for MemoryProjectRepository {
    fn create(&self, project: Project) -> ProjectResult<Project> {
        self.projects
            .write()
            .expect("project memory repository poisoned")
            .insert(project.id.clone(), project.clone());
        Ok(project)
    }

    fn update(&self, project: Project) -> ProjectResult<Project> {
        self.projects
            .write()
            .expect("project memory repository poisoned")
            .insert(project.id.clone(), project.clone());
        Ok(project)
    }

    fn delete(&self, id: &str) -> ProjectResult<Option<Project>> {
        Ok(self
            .projects
            .write()
            .expect("project memory repository poisoned")
            .remove(id))
    }

    fn find_by_id(&self, id: &str) -> ProjectResult<Option<Project>> {
        Ok(self
            .projects
            .read()
            .expect("project memory repository poisoned")
            .get(id)
            .cloned())
    }

    fn list(&self) -> ProjectResult<Vec<Project>> {
        Ok(self
            .projects
            .read()
            .expect("project memory repository poisoned")
            .values()
            .cloned()
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct SqliteProjectRepository {
    path: PathBuf,
}

impl SqliteProjectRepository {
    pub fn open(path: impl Into<PathBuf>) -> ProjectResult<Self> {
        let repository = Self { path: path.into() };
        repository.ensure_schema()?;
        Ok(repository)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn connection(&self) -> ProjectResult<Connection> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Connection::open(&self.path)?)
    }

    fn ensure_schema(&self) -> ProjectResult<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                icon TEXT NOT NULL DEFAULT 'package',
                color TEXT NOT NULL DEFAULT 'blue',
                template TEXT NOT NULL DEFAULT '"blank"',
                tunnel_ids TEXT NOT NULL DEFAULT '[]',
                domains TEXT NOT NULL DEFAULT '[]',
                certificate_ids TEXT NOT NULL DEFAULT '[]',
                tags TEXT NOT NULL DEFAULT '[]',
                environments TEXT NOT NULL DEFAULT '[]',
                notes TEXT NOT NULL DEFAULT '[]',
                favorite INTEGER NOT NULL DEFAULT 0,
                pinned INTEGER NOT NULL DEFAULT 0,
                auto_start INTEGER NOT NULL DEFAULT 0,
                startup_policy TEXT,
                last_activity_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_projects_updated_at ON projects(updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_projects_last_activity_at ON projects(last_activity_at DESC);
            PRAGMA user_version = 1;
            "#,
        )?;
        Ok(())
    }

    fn upsert(&self, project: &Project) -> ProjectResult<()> {
        let connection = self.connection()?;
        connection.execute(
            r#"
            INSERT INTO projects (
                id,
                name,
                description,
                icon,
                color,
                template,
                tunnel_ids,
                domains,
                certificate_ids,
                tags,
                environments,
                notes,
                favorite,
                pinned,
                auto_start,
                startup_policy,
                last_activity_at,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                icon = excluded.icon,
                color = excluded.color,
                template = excluded.template,
                tunnel_ids = excluded.tunnel_ids,
                domains = excluded.domains,
                certificate_ids = excluded.certificate_ids,
                tags = excluded.tags,
                environments = excluded.environments,
                notes = excluded.notes,
                favorite = excluded.favorite,
                pinned = excluded.pinned,
                auto_start = excluded.auto_start,
                startup_policy = excluded.startup_policy,
                last_activity_at = excluded.last_activity_at,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at
            "#,
            params![
                project.id,
                project.name,
                project.description,
                project.icon,
                project.color,
                json_string(&project.template)?,
                json_string(&project.tunnel_ids)?,
                json_string(&project.domains)?,
                json_string(&project.certificate_ids)?,
                json_string(&project.tags)?,
                json_string(&project.environments)?,
                json_string(&project.notes)?,
                bool_to_i64(project.favorite),
                bool_to_i64(project.pinned),
                bool_to_i64(project.auto_start),
                project.startup_policy,
                project.last_activity_at,
                project.created_at,
                project.updated_at,
            ],
        )?;
        Ok(())
    }
}

impl ProjectRepository for SqliteProjectRepository {
    fn create(&self, project: Project) -> ProjectResult<Project> {
        self.upsert(&project)?;
        Ok(project)
    }

    fn update(&self, project: Project) -> ProjectResult<Project> {
        self.upsert(&project)?;
        Ok(project)
    }

    fn delete(&self, id: &str) -> ProjectResult<Option<Project>> {
        let project = self.find_by_id(id)?;
        if project.is_some() {
            let connection = self.connection()?;
            connection.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        }
        Ok(project)
    }

    fn find_by_id(&self, id: &str) -> ProjectResult<Option<Project>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(project_select_sql("WHERE id = ?1").as_str())?;
        let row = statement
            .query_row(params![id], read_project_row)
            .optional()?;
        row.map(project_from_row).transpose()
    }

    fn list(&self) -> ProjectResult<Vec<Project>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(project_select_sql("").as_str())?;
        let rows = statement.query_map([], read_project_row)?;
        let mut projects = Vec::new();

        for row in rows {
            projects.push(project_from_row(row?)?);
        }

        projects.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_activity_at.cmp(&a.last_activity_at))
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(projects)
    }
}

#[derive(Debug)]
struct ProjectRow {
    id: String,
    name: String,
    description: String,
    icon: String,
    color: String,
    template: String,
    tunnel_ids: String,
    domains: String,
    certificate_ids: String,
    tags: String,
    environments: String,
    notes: String,
    favorite: i64,
    pinned: i64,
    auto_start: i64,
    startup_policy: Option<String>,
    last_activity_at: i64,
    created_at: i64,
    updated_at: i64,
}

fn project_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            id,
            name,
            description,
            icon,
            color,
            template,
            tunnel_ids,
            domains,
            certificate_ids,
            tags,
            environments,
            notes,
            favorite,
            pinned,
            auto_start,
            startup_policy,
            last_activity_at,
            created_at,
            updated_at
        FROM projects
        {where_clause}
        ORDER BY pinned DESC, last_activity_at DESC, name ASC
        "#
    )
}

fn read_project_row(row: &Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        color: row.get(4)?,
        template: row.get(5)?,
        tunnel_ids: row.get(6)?,
        domains: row.get(7)?,
        certificate_ids: row.get(8)?,
        tags: row.get(9)?,
        environments: row.get(10)?,
        notes: row.get(11)?,
        favorite: row.get(12)?,
        pinned: row.get(13)?,
        auto_start: row.get(14)?,
        startup_policy: row.get(15)?,
        last_activity_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

fn project_from_row(row: ProjectRow) -> ProjectResult<Project> {
    Ok(Project {
        id: row.id,
        name: row.name,
        description: row.description,
        icon: row.icon,
        color: row.color,
        template: json_value::<ProjectTemplate>(&row.template)?,
        tunnel_ids: json_value::<Vec<String>>(&row.tunnel_ids)?,
        domains: json_value::<Vec<String>>(&row.domains)?,
        certificate_ids: json_value::<Vec<String>>(&row.certificate_ids)?,
        tags: json_value::<Vec<String>>(&row.tags)?,
        environments: json_value::<Vec<ProjectEnvironment>>(&row.environments)?,
        notes: json_value::<Vec<ProjectNote>>(&row.notes)?,
        favorite: row.favorite != 0,
        pinned: row.pinned != 0,
        auto_start: row.auto_start != 0,
        startup_policy: row.startup_policy,
        last_activity_at: row.last_activity_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn json_string<T: Serialize>(value: &T) -> ProjectResult<String> {
    Ok(serde_json::to_string(value)?)
}

fn json_value<T: DeserializeOwned>(value: &str) -> ProjectResult<T> {
    Ok(serde_json::from_str(value)?)
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::project::entity::CreateProjectRequest;

    #[test]
    fn memory_repository_roundtrips_project() -> anyhow::Result<()> {
        let repository = MemoryProjectRepository::new();
        let mut project = Project::new(CreateProjectRequest {
            name: "Gate Workspace".to_string(),
            tags: Some(vec!["dev".to_string()]),
            ..Default::default()
        });
        project.add_tunnel("tunnel-1");

        repository.create(project.clone())?;
        let loaded = repository
            .find_by_id(&project.id)?
            .expect("project should be present");

        assert_eq!(loaded.name, "Gate Workspace");
        assert_eq!(loaded.tunnel_ids, vec!["tunnel-1"]);
        Ok(())
    }

    #[test]
    fn sqlite_repository_persists_and_deletes_project() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("projects.sqlite3");
        let repository = SqliteProjectRepository::open(&db_path)?;
        let project = Project::new(CreateProjectRequest {
            name: "Persisted".to_string(),
            domains: Some(vec!["example.test".to_string()]),
            template: Some(ProjectTemplate::Webhook),
            ..Default::default()
        });
        let project_id = project.id.clone();

        repository.create(project)?;
        let reopened = SqliteProjectRepository::open(&db_path)?;
        let loaded = reopened
            .find_by_id(&project_id)?
            .expect("project should survive reopen");
        assert_eq!(loaded.domains, vec!["example.test"]);
        assert_eq!(loaded.template, ProjectTemplate::Webhook);

        let deleted = reopened.delete(&project_id)?;
        assert!(deleted.is_some());
        assert!(SqliteProjectRepository::open(&db_path)?
            .find_by_id(&project_id)?
            .is_none());

        Ok(())
    }
}
