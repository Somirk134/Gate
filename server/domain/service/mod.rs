use crate::config::DomainConfig;
use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::model::{Alias, Domain, DomainId, DomainStatus, Host, RecordType, TunnelId};
use crate::repository::DomainRepository;
use crate::validator::DomainValidator;

#[derive(Clone, Debug)]
pub struct CreateDomainRequest {
    pub id: DomainId,
    pub host: String,
    pub aliases: Vec<String>,
    pub record_type: RecordType,
}

#[derive(Clone, Debug)]
pub struct UpdateDomainRequest {
    pub id: DomainId,
    pub host: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub record_type: Option<RecordType>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchDomainRequest {
    pub keyword: Option<String>,
    pub tunnel_id: Option<TunnelId>,
    pub status: Option<DomainStatus>,
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainServiceOutput<T> {
    pub value: T,
    pub event: Option<DomainEvent>,
}

impl<T> DomainServiceOutput<T> {
    pub fn with_event(value: T, event: DomainEvent) -> Self {
        Self {
            value,
            event: Some(event),
        }
    }

    pub fn without_event(value: T) -> Self {
        Self { value, event: None }
    }
}

#[derive(Clone)]
pub struct DomainService<R, V>
where
    R: DomainRepository,
    V: DomainValidator,
{
    repository: R,
    validator: V,
    config: DomainConfig,
}

impl<R, V> DomainService<R, V>
where
    R: DomainRepository,
    V: DomainValidator,
{
    pub fn builder() -> DomainServiceBuilder<R, V> {
        DomainServiceBuilder::default()
    }

    pub fn new(repository: R, validator: V, config: DomainConfig) -> Self {
        Self {
            repository,
            validator,
            config,
        }
    }

    pub fn create_domain(
        &self,
        request: CreateDomainRequest,
    ) -> Result<DomainServiceOutput<Domain>, DomainError> {
        if self.repository.list()?.len() >= self.config.max_domains {
            return Err(DomainError::LimitExceeded {
                max: self.config.max_domains,
            });
        }

        let host = self.validator.validate_host(&request.host, &self.config)?;
        self.validator.validate_unique(&self.repository, &host)?;
        let aliases = self.validate_aliases(&request.aliases)?;

        let domain = Domain::builder(request.id, host)
            .aliases(aliases)
            .record_type(request.record_type)
            .build()?;
        let domain = self.repository.create(domain)?;
        Ok(DomainServiceOutput::with_event(
            domain.clone(),
            DomainEvent::created(&domain),
        ))
    }

    pub fn delete_domain(&self, id: &DomainId) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let domain = self.repository.delete(id)?;
        Ok(DomainServiceOutput::with_event(
            domain.clone(),
            DomainEvent::deleted(&domain),
        ))
    }

    pub fn update_domain(
        &self,
        request: UpdateDomainRequest,
    ) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let mut domain = self
            .repository
            .find_by_id(&request.id)?
            .ok_or_else(|| DomainError::NotFound(request.id.to_string()))?;

        if let Some(host) = request.host {
            let host = self.validate_rename_host(domain.id(), &host)?;
            domain.rename(host);
        }

        if let Some(aliases) = request.aliases {
            domain.replace_aliases(self.validate_aliases(&aliases)?)?;
        }

        if let Some(record_type) = request.record_type {
            domain.update_record_type(record_type);
        }

        let domain = self.repository.update(domain)?;
        Ok(DomainServiceOutput::with_event(
            domain.clone(),
            DomainEvent::updated(&domain),
        ))
    }

    pub fn bind_tunnel(
        &self,
        id: &DomainId,
        tunnel_id: TunnelId,
    ) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let domain = self.repository.bind_tunnel(id, tunnel_id.clone())?;
        Ok(DomainServiceOutput::with_event(
            domain,
            DomainEvent::DomainBound {
                domain_id: id.clone(),
                tunnel_id,
            },
        ))
    }

    pub fn unbind_tunnel(&self, id: &DomainId) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let domain = self.repository.unbind_tunnel(id)?;
        Ok(DomainServiceOutput::with_event(
            domain,
            DomainEvent::DomainUnbound {
                domain_id: id.clone(),
            },
        ))
    }

    pub fn enable(&self, id: &DomainId) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let mut domain = self
            .repository
            .find_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(id.to_string()))?;
        domain.enable();
        let domain = self.repository.update(domain)?;
        Ok(DomainServiceOutput::with_event(
            domain,
            DomainEvent::DomainEnabled {
                domain_id: id.clone(),
            },
        ))
    }

    pub fn disable(&self, id: &DomainId) -> Result<DomainServiceOutput<Domain>, DomainError> {
        let mut domain = self
            .repository
            .find_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(id.to_string()))?;
        domain.disable();
        let domain = self.repository.update(domain)?;
        Ok(DomainServiceOutput::with_event(
            domain,
            DomainEvent::DomainDisabled {
                domain_id: id.clone(),
            },
        ))
    }

    pub fn rename(
        &self,
        id: &DomainId,
        host: &str,
    ) -> Result<DomainServiceOutput<Domain>, DomainError> {
        self.update_domain(UpdateDomainRequest {
            id: id.clone(),
            host: Some(host.to_string()),
            aliases: None,
            record_type: None,
        })
    }

    pub fn list(&self) -> Result<DomainServiceOutput<Vec<Domain>>, DomainError> {
        Ok(DomainServiceOutput::without_event(self.repository.list()?))
    }

    pub fn search(
        &self,
        request: SearchDomainRequest,
    ) -> Result<DomainServiceOutput<Vec<Domain>>, DomainError> {
        let keyword = request
            .keyword
            .as_ref()
            .map(|value| value.to_ascii_lowercase());
        let domains = self
            .repository
            .list()?
            .into_iter()
            .filter(|domain| matches_keyword(domain, keyword.as_deref()))
            .filter(|domain| matches_tunnel(domain, request.tunnel_id.as_ref()))
            .filter(|domain| matches_status(domain, request.status.as_ref()))
            .filter(|domain| matches_enabled(domain, request.enabled))
            .collect();

        Ok(DomainServiceOutput::without_event(domains))
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    fn validate_aliases(&self, aliases: &[String]) -> Result<Vec<Alias>, DomainError> {
        aliases
            .iter()
            .map(|alias| Ok(self.validator.validate_alias(alias, &self.config)?))
            .collect()
    }

    fn validate_rename_host(&self, current_id: &DomainId, host: &str) -> Result<Host, DomainError> {
        let host = self.validator.validate_host(host, &self.config)?;
        if let Some(existing) = self.repository.find_by_host(&host)? {
            if existing.id() != current_id {
                return Err(DomainError::AlreadyExists(host.to_string()));
            }
        }
        Ok(host)
    }
}

#[derive(Clone)]
pub struct DomainServiceBuilder<R, V>
where
    R: DomainRepository,
    V: DomainValidator,
{
    repository: Option<R>,
    validator: Option<V>,
    config: DomainConfig,
}

impl<R, V> DomainServiceBuilder<R, V>
where
    R: DomainRepository,
    V: DomainValidator,
{
    pub fn repository(mut self, repository: R) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn validator(mut self, validator: V) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn config(mut self, config: DomainConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<DomainService<R, V>, DomainError> {
        let repository = self
            .repository
            .ok_or_else(|| DomainError::InvalidOperation("repository is required".to_string()))?;
        let validator = self
            .validator
            .ok_or_else(|| DomainError::InvalidOperation("validator is required".to_string()))?;
        Ok(DomainService::new(repository, validator, self.config))
    }
}

impl<R, V> Default for DomainServiceBuilder<R, V>
where
    R: DomainRepository,
    V: DomainValidator,
{
    fn default() -> Self {
        Self {
            repository: None,
            validator: None,
            config: DomainConfig::default(),
        }
    }
}

fn matches_keyword(domain: &Domain, keyword: Option<&str>) -> bool {
    match keyword {
        Some(value) => {
            domain.host().as_str().contains(value)
                || domain
                    .aliases()
                    .iter()
                    .any(|alias| alias.as_str().contains(value))
        }
        None => true,
    }
}

fn matches_tunnel(domain: &Domain, tunnel_id: Option<&TunnelId>) -> bool {
    match tunnel_id {
        Some(expected) => domain.tunnel_id() == Some(expected),
        None => true,
    }
}

fn matches_status(domain: &Domain, status: Option<&DomainStatus>) -> bool {
    match status {
        Some(expected) => domain.status() == expected,
        None => true,
    }
}

fn matches_enabled(domain: &Domain, enabled: Option<bool>) -> bool {
    match enabled {
        Some(expected) => domain.is_enabled() == expected,
        None => true,
    }
}
