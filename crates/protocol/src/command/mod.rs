use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// Unified command namespace. Serialized representation is the dotted command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    ProjectCreate,
    ProjectUpdate,
    ProjectDelete,
    ProjectList,
    TunnelCreate,
    TunnelRegister,
    TunnelStart,
    TunnelStop,
    TunnelRestart,
    TunnelStatistics,
    TunnelRelayConnect,
    TunnelRelayStart,
    DomainCreate,
    DomainBind,
    DomainUnbind,
    DomainDelete,
    ServerConnect,
    ServerDisconnect,
    ServerStatus,
    LogSubscribe,
    LogUnsubscribe,
    LogQuery,
    StatisticsQuery,
    SettingsGet,
    SettingsSet,
    HeartbeatPing,
    HeartbeatPong,
    AuthLogin,
    AuthLogout,
    AuthRefresh,
    SystemVersion,
    SystemHealth,
    SystemShutdown,
    Custom(String),
}

impl Command {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ProjectCreate => "project.create",
            Self::ProjectUpdate => "project.update",
            Self::ProjectDelete => "project.delete",
            Self::ProjectList => "project.list",
            Self::TunnelCreate => "tunnel.create",
            Self::TunnelRegister => "tunnel.register",
            Self::TunnelStart => "tunnel.start",
            Self::TunnelStop => "tunnel.stop",
            Self::TunnelRestart => "tunnel.restart",
            Self::TunnelStatistics => "tunnel.statistics",
            Self::TunnelRelayConnect => "tunnel.relay.connect",
            Self::TunnelRelayStart => "tunnel.relay.start",
            Self::DomainCreate => "domain.create",
            Self::DomainBind => "domain.bind",
            Self::DomainUnbind => "domain.unbind",
            Self::DomainDelete => "domain.delete",
            Self::ServerConnect => "server.connect",
            Self::ServerDisconnect => "server.disconnect",
            Self::ServerStatus => "server.status",
            Self::LogSubscribe => "log.subscribe",
            Self::LogUnsubscribe => "log.unsubscribe",
            Self::LogQuery => "log.query",
            Self::StatisticsQuery => "statistics.query",
            Self::SettingsGet => "settings.get",
            Self::SettingsSet => "settings.set",
            Self::HeartbeatPing => "heartbeat.ping",
            Self::HeartbeatPong => "heartbeat.pong",
            Self::AuthLogin => "auth.login",
            Self::AuthLogout => "auth.logout",
            Self::AuthRefresh => "auth.refresh",
            Self::SystemVersion => "system.version",
            Self::SystemHealth => "system.health",
            Self::SystemShutdown => "system.shutdown",
            Self::Custom(value) => value.as_str(),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Command {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "project.create" => Self::ProjectCreate,
            "project.update" => Self::ProjectUpdate,
            "project.delete" => Self::ProjectDelete,
            "project.list" => Self::ProjectList,
            "tunnel.create" => Self::TunnelCreate,
            "tunnel.register" => Self::TunnelRegister,
            "tunnel.start" => Self::TunnelStart,
            "tunnel.stop" => Self::TunnelStop,
            "tunnel.restart" => Self::TunnelRestart,
            "tunnel.statistics" => Self::TunnelStatistics,
            "tunnel.relay.connect" => Self::TunnelRelayConnect,
            "tunnel.relay.start" => Self::TunnelRelayStart,
            "domain.create" => Self::DomainCreate,
            "domain.bind" => Self::DomainBind,
            "domain.unbind" => Self::DomainUnbind,
            "domain.delete" => Self::DomainDelete,
            "server.connect" => Self::ServerConnect,
            "server.disconnect" => Self::ServerDisconnect,
            "server.status" => Self::ServerStatus,
            "log.subscribe" => Self::LogSubscribe,
            "log.unsubscribe" => Self::LogUnsubscribe,
            "log.query" => Self::LogQuery,
            "statistics.query" => Self::StatisticsQuery,
            "settings.get" => Self::SettingsGet,
            "settings.set" => Self::SettingsSet,
            "heartbeat.ping" => Self::HeartbeatPing,
            "heartbeat.pong" => Self::HeartbeatPong,
            "auth.login" => Self::AuthLogin,
            "auth.logout" => Self::AuthLogout,
            "auth.refresh" => Self::AuthRefresh,
            "system.version" => Self::SystemVersion,
            "system.health" => Self::SystemHealth,
            "system.shutdown" => Self::SystemShutdown,
            other => Self::Custom(other.to_owned()),
        })
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Command::from_str(&value).map_err(D::Error::custom)
    }
}
