#[path = "../mod.rs"]
mod domain;

pub use domain::{config, error, event, model, repository, resolver, service, storage, traits, validator};

use domain::config::DomainConfig;
use domain::validator::{DomainValidator, RfcDomainValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = RfcDomainValidator::new();
    let host = validator.validate_host("api.gate.dev", &DomainConfig::default())?;
    println!("valid host: {host}");
    Ok(())
}
