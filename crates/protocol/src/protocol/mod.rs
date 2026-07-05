mod state;

use std::{collections::BTreeMap, sync::Arc};

pub use state::{AuthenticationState, ConnectionState, ProtocolState, TunnelState};

use crate::{
    codec::{Codec, JsonCodec},
    error::ProtocolError,
    message::{Message, Metadata},
    shared::{ClientInfo, ServerInfo, VersionInfo},
    version::{ProtocolVersion, VersionNegotiation},
};

/// Protocol boundary. Implementations validate and encode only protocol data.
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;

    fn version(&self) -> ProtocolVersion;

    fn codec(&self) -> &dyn Codec;

    fn validate(&self, message: &Message) -> Result<(), ProtocolError>;
}

/// V1 protocol implementation backed by JSON.
pub struct GateProtocol {
    version: ProtocolVersion,
    codec: Box<dyn Codec>,
}

impl GateProtocol {
    pub fn v1_json() -> Self {
        Self {
            version: ProtocolVersion::V1,
            codec: Box::new(JsonCodec::new()),
        }
    }
}

impl Default for GateProtocol {
    fn default() -> Self {
        Self::v1_json()
    }
}

impl Protocol for GateProtocol {
    fn name(&self) -> &'static str {
        "gate-protocol"
    }

    fn version(&self) -> ProtocolVersion {
        self.version
    }

    fn codec(&self) -> &dyn Codec {
        self.codec.as_ref()
    }

    fn validate(&self, message: &Message) -> Result<(), ProtocolError> {
        if message.header.protocol_version.major != self.version.major {
            return Err(ProtocolError::Version(
                crate::error::VersionError::UnsupportedVersion {
                    requested: message.header.protocol_version,
                },
            ));
        }
        Ok(())
    }
}

/// Runtime context shared by protocol managers and transport adapters.
#[derive(Debug, Clone)]
pub struct ProtocolContext {
    pub client: Option<ClientInfo>,
    pub server: Option<ServerInfo>,
    pub versions: VersionInfo,
    pub metadata: Metadata,
    pub connection_state: ConnectionState,
    pub protocol_state: ProtocolState,
    pub authentication_state: AuthenticationState,
    pub tunnel_state: TunnelState,
}

impl Default for ProtocolContext {
    fn default() -> Self {
        Self {
            client: None,
            server: None,
            versions: VersionInfo::default(),
            metadata: Metadata::default(),
            connection_state: ConnectionState::Disconnected,
            protocol_state: ProtocolState::Initialized,
            authentication_state: AuthenticationState::Anonymous,
            tunnel_state: TunnelState::Idle,
        }
    }
}

/// Registry for protocol versions and their concrete codecs.
#[derive(Default)]
pub struct ProtocolRegistry {
    protocols: BTreeMap<ProtocolVersion, Arc<dyn Protocol>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, protocol: P)
    where
        P: Protocol + 'static,
    {
        self.protocols
            .insert(protocol.version(), Arc::new(protocol));
    }

    pub fn get(&self, version: ProtocolVersion) -> Option<Arc<dyn Protocol>> {
        self.protocols.get(&version).cloned()
    }

    pub fn latest(&self) -> Option<Arc<dyn Protocol>> {
        self.protocols.iter().next_back().map(|(_, protocol)| protocol.clone())
    }

    pub fn supported_versions(&self) -> Vec<ProtocolVersion> {
        self.protocols.keys().copied().collect()
    }
}

/// Central protocol facade used by client and server code.
pub struct ProtocolManager {
    registry: ProtocolRegistry,
    context: ProtocolContext,
    active_version: ProtocolVersion,
}

impl ProtocolManager {
    pub fn new(
        registry: ProtocolRegistry,
        context: ProtocolContext,
        active_version: ProtocolVersion,
    ) -> Self {
        Self {
            registry,
            context,
            active_version,
        }
    }

    pub fn context(&self) -> &ProtocolContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut ProtocolContext {
        &mut self.context
    }

    pub fn active_version(&self) -> ProtocolVersion {
        self.active_version
    }

    pub fn protocol(&self) -> Result<Arc<dyn Protocol>, ProtocolError> {
        self.registry
            .get(self.active_version)
            .ok_or_else(|| crate::error::VersionError::UnsupportedVersion {
                requested: self.active_version,
            })
            .map_err(ProtocolError::from)
    }

    pub fn encode(&self, message: &Message) -> Result<Vec<u8>, ProtocolError> {
        let protocol = self.protocol()?;
        protocol.validate(message)?;
        protocol.codec().encode(message).map_err(ProtocolError::from)
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<Message, ProtocolError> {
        let protocol = self.protocol()?;
        let message = protocol.codec().decode(bytes)?;
        protocol.validate(&message)?;
        Ok(message)
    }

    pub fn negotiate(
        &mut self,
        negotiation: &VersionNegotiation,
        remote: &[ProtocolVersion],
    ) -> Result<ProtocolVersion, ProtocolError> {
        let selected = negotiation.negotiate(remote)?;
        if self.registry.get(selected).is_none() {
            return Err(crate::error::VersionError::UnsupportedVersion {
                requested: selected,
            }
            .into());
        }
        self.active_version = selected;
        Ok(selected)
    }
}

/// Builder for a protocol manager with V1 JSON registered by default.
pub struct ProtocolBuilder {
    registry: ProtocolRegistry,
    context: ProtocolContext,
    active_version: ProtocolVersion,
}

impl ProtocolBuilder {
    pub fn new() -> Self {
        let mut registry = ProtocolRegistry::new();
        registry.register(GateProtocol::v1_json());

        Self {
            registry,
            context: ProtocolContext::default(),
            active_version: ProtocolVersion::V1,
        }
    }

    pub fn context(mut self, context: ProtocolContext) -> Self {
        self.context = context;
        self
    }

    pub fn register<P>(mut self, protocol: P) -> Self
    where
        P: Protocol + 'static,
    {
        self.registry.register(protocol);
        self
    }

    pub fn active_version(mut self, version: ProtocolVersion) -> Self {
        self.active_version = version;
        self
    }

    pub fn build(self) -> ProtocolManager {
        ProtocolManager::new(self.registry, self.context, self.active_version)
    }
}

impl Default for ProtocolBuilder {
    fn default() -> Self {
        Self::new()
    }
}
