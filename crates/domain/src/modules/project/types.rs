use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectTemplate {
    Blank,
    SpringBoot,
    Vue,
    Node,
    Python,
    Docker,
    McpServer,
    AiAgent,
    Nas,
    Ssh,
    Git,
    Webhook,
    Custom,
}

impl Default for ProjectTemplate {
    fn default() -> Self {
        Self::Blank
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelRecommendation {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub local_host: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplateProfile {
    pub key: ProjectTemplate,
    pub label: String,
    pub icon: String,
    pub color: String,
    pub description: String,
    pub tags: Vec<String>,
    pub recommendations: Vec<TunnelRecommendation>,
}

pub fn project_template_profiles() -> Vec<ProjectTemplateProfile> {
    use ProjectTemplate::*;

    vec![
        profile(
            Blank,
            "project.templates.blank.label",
            "package",
            "blue",
            "project.templates.blank.description",
            vec![],
            vec![],
        ),
        profile(
            SpringBoot,
            "project.templates.springBoot.label",
            "coffee",
            "green",
            "project.templates.springBoot.description",
            vec!["backend", "java"],
            vec![recommendation(
                "spring-http",
                "project.templateRecommendations.spring-http.name",
                "http",
                8080,
                18080,
                "project.templateRecommendations.spring-http.description",
                vec!["backend", "http"],
            )],
        ),
        profile(
            Vue,
            "project.templates.vue.label",
            "code",
            "cyan",
            "project.templates.vue.description",
            vec!["frontend", "vite"],
            vec![recommendation(
                "vue-dev",
                "project.templateRecommendations.vue-dev.name",
                "http",
                5173,
                15173,
                "project.templateRecommendations.vue-dev.description",
                vec!["frontend", "http"],
            )],
        ),
        profile(
            Node,
            "project.templates.node.label",
            "terminal",
            "green",
            "project.templates.node.description",
            vec!["node", "api"],
            vec![recommendation(
                "node-api",
                "project.templateRecommendations.node-api.name",
                "http",
                3000,
                13000,
                "project.templateRecommendations.node-api.description",
                vec!["node", "http"],
            )],
        ),
        profile(
            Python,
            "project.templates.python.label",
            "file-code",
            "blue",
            "project.templates.python.description",
            vec!["python", "api"],
            vec![recommendation(
                "python-api",
                "project.templateRecommendations.python-api.name",
                "http",
                8000,
                18000,
                "project.templateRecommendations.python-api.description",
                vec!["python", "http"],
            )],
        ),
        profile(
            Docker,
            "project.templates.docker.label",
            "box",
            "indigo",
            "project.templates.docker.description",
            vec!["docker"],
            vec![recommendation(
                "docker-web",
                "project.templateRecommendations.docker-web.name",
                "http",
                8080,
                18081,
                "project.templateRecommendations.docker-web.description",
                vec!["docker", "http"],
            )],
        ),
        profile(
            McpServer,
            "project.templates.mcpServer.label",
            "plug-zap",
            "purple",
            "project.templates.mcpServer.description",
            vec!["mcp", "ai"],
            vec![recommendation(
                "mcp-server",
                "project.templateRecommendations.mcp-server.name",
                "http",
                3333,
                13333,
                "project.templateRecommendations.mcp-server.description",
                vec!["mcp", "ai"],
            )],
        ),
        profile(
            AiAgent,
            "project.templates.aiAgent.label",
            "sparkles",
            "pink",
            "project.templates.aiAgent.description",
            vec!["ai", "agent"],
            vec![recommendation(
                "agent-console",
                "project.templateRecommendations.agent-console.name",
                "http",
                7860,
                17860,
                "project.templateRecommendations.agent-console.description",
                vec!["ai", "agent"],
            )],
        ),
        profile(
            Nas,
            "project.templates.nas.label",
            "hard-drive",
            "slate",
            "project.templates.nas.description",
            vec!["home", "storage"],
            vec![recommendation(
                "nas-panel",
                "project.templateRecommendations.nas-panel.name",
                "http",
                5000,
                15000,
                "project.templateRecommendations.nas-panel.description",
                vec!["nas", "home"],
            )],
        ),
        profile(
            Ssh,
            "project.templates.ssh.label",
            "terminal",
            "amber",
            "project.templates.ssh.description",
            vec!["ssh"],
            vec![recommendation(
                "ssh",
                "project.templateRecommendations.ssh.name",
                "tcp",
                22,
                10022,
                "project.templateRecommendations.ssh.description",
                vec!["ssh", "tcp"],
            )],
        ),
        profile(
            Git,
            "project.templates.git.label",
            "git-branch",
            "orange",
            "project.templates.git.description",
            vec!["git"],
            vec![recommendation(
                "git-http",
                "project.templateRecommendations.git-http.name",
                "http",
                3000,
                13001,
                "project.templateRecommendations.git-http.description",
                vec!["git", "http"],
            )],
        ),
        profile(
            Webhook,
            "project.templates.webhook.label",
            "radio",
            "teal",
            "project.templates.webhook.description",
            vec!["webhook"],
            vec![recommendation(
                "webhook",
                "project.templateRecommendations.webhook.name",
                "http",
                9000,
                19000,
                "project.templateRecommendations.webhook.description",
                vec!["webhook", "http"],
            )],
        ),
        profile(
            Custom,
            "project.templates.custom.label",
            "layers",
            "blue",
            "project.templates.custom.description",
            vec!["custom"],
            vec![recommendation(
                "custom-http",
                "project.templateRecommendations.custom-http.name",
                "http",
                8080,
                18082,
                "project.templateRecommendations.custom-http.description",
                vec!["custom"],
            )],
        ),
    ]
}

pub fn recommendations_for_template(template: ProjectTemplate) -> Vec<TunnelRecommendation> {
    project_template_profiles()
        .into_iter()
        .find(|profile| profile.key == template)
        .map(|profile| profile.recommendations)
        .unwrap_or_default()
}

fn profile(
    key: ProjectTemplate,
    label: &str,
    icon: &str,
    color: &str,
    description: &str,
    tags: Vec<&str>,
    recommendations: Vec<TunnelRecommendation>,
) -> ProjectTemplateProfile {
    ProjectTemplateProfile {
        key,
        label: label.to_string(),
        icon: icon.to_string(),
        color: color.to_string(),
        description: description.to_string(),
        tags: tags.into_iter().map(str::to_string).collect(),
        recommendations,
    }
}

fn recommendation(
    id: &str,
    name: &str,
    protocol: &str,
    local_port: u16,
    remote_port: u16,
    description: &str,
    tags: Vec<&str>,
) -> TunnelRecommendation {
    TunnelRecommendation {
        id: id.to_string(),
        name: name.to_string(),
        protocol: protocol.to_string(),
        local_host: "127.0.0.1".to_string(),
        local_port,
        remote_port,
        description: description.to_string(),
        tags: tags.into_iter().map(str::to_string).collect(),
    }
}
