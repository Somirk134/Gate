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
            "Blank",
            "package",
            "blue",
            "空白工作空间",
            vec![],
            vec![],
        ),
        profile(
            SpringBoot,
            "SpringBoot",
            "coffee",
            "green",
            "Java Web 服务，默认推荐 HTTP 8080",
            vec!["Backend", "Java"],
            vec![recommendation(
                "spring-http",
                "Spring Boot HTTP",
                "http",
                8080,
                18080,
                "暴露 Spring Boot 本地 HTTP 服务",
                vec!["Backend", "HTTP"],
            )],
        ),
        profile(
            Vue,
            "Vue",
            "code",
            "cyan",
            "前端开发服务器，默认推荐 HTTP 5173",
            vec!["Frontend", "Vite"],
            vec![recommendation(
                "vue-dev",
                "Vue Dev Server",
                "http",
                5173,
                15173,
                "暴露 Vite/Vue 开发服务器",
                vec!["Frontend", "HTTP"],
            )],
        ),
        profile(
            Node,
            "Node",
            "terminal",
            "green",
            "Node.js API 或本地服务",
            vec!["Node", "API"],
            vec![recommendation(
                "node-api",
                "Node API",
                "http",
                3000,
                13000,
                "暴露 Node.js HTTP API",
                vec!["Node", "HTTP"],
            )],
        ),
        profile(
            Python,
            "Python",
            "file-code",
            "blue",
            "Python Flask/FastAPI 服务",
            vec!["Python", "API"],
            vec![recommendation(
                "python-api",
                "Python API",
                "http",
                8000,
                18000,
                "暴露 Python Web 服务",
                vec!["Python", "HTTP"],
            )],
        ),
        profile(
            Docker,
            "Docker",
            "box",
            "indigo",
            "容器化服务入口",
            vec!["Docker"],
            vec![recommendation(
                "docker-web",
                "Docker Web",
                "http",
                8080,
                18081,
                "暴露容器映射的 Web 服务",
                vec!["Docker", "HTTP"],
            )],
        ),
        profile(
            McpServer,
            "MCP Server",
            "plug-zap",
            "purple",
            "本地 MCP 工具服务",
            vec!["MCP", "AI"],
            vec![recommendation(
                "mcp-server",
                "MCP Server",
                "http",
                3333,
                13333,
                "暴露 MCP HTTP/SSE 服务",
                vec!["MCP", "AI"],
            )],
        ),
        profile(
            AiAgent,
            "AI Agent",
            "sparkles",
            "pink",
            "Agent 调试、Webhook 或 Web UI",
            vec!["AI", "Agent"],
            vec![recommendation(
                "agent-console",
                "Agent Console",
                "http",
                7860,
                17860,
                "暴露 Agent 控制台或调试端点",
                vec!["AI", "Agent"],
            )],
        ),
        profile(
            Nas,
            "NAS",
            "hard-drive",
            "slate",
            "家庭 NAS、媒体与管理端口",
            vec!["Home", "Storage"],
            vec![recommendation(
                "nas-panel",
                "NAS Panel",
                "http",
                5000,
                15000,
                "暴露 NAS 管理面板",
                vec!["NAS", "Home"],
            )],
        ),
        profile(
            Ssh,
            "SSH",
            "terminal",
            "amber",
            "远程 shell 或设备维护",
            vec!["SSH"],
            vec![recommendation(
                "ssh",
                "SSH",
                "tcp",
                22,
                10022,
                "暴露 SSH TCP 端口",
                vec!["SSH", "TCP"],
            )],
        ),
        profile(
            Git,
            "Git",
            "git-branch",
            "orange",
            "Git 服务与代码托管",
            vec!["Git"],
            vec![recommendation(
                "git-http",
                "Git HTTP",
                "http",
                3000,
                13001,
                "暴露 Git Web 面板",
                vec!["Git", "HTTP"],
            )],
        ),
        profile(
            Webhook,
            "Webhook",
            "radio",
            "teal",
            "本地 Webhook 回调调试",
            vec!["Webhook"],
            vec![recommendation(
                "webhook",
                "Webhook Receiver",
                "http",
                9000,
                19000,
                "暴露本地 Webhook 接收端",
                vec!["Webhook", "HTTP"],
            )],
        ),
        profile(
            Custom,
            "Custom",
            "layers",
            "blue",
            "自定义资源集合",
            vec!["Custom"],
            vec![recommendation(
                "custom-http",
                "Custom HTTP",
                "http",
                8080,
                18082,
                "自定义 HTTP 服务",
                vec!["Custom"],
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
