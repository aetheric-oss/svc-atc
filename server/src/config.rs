//! # Config
//!
//! Define and implement config options for module

use anyhow::Result;
use config::{ConfigError, Environment};
use dotenv::dotenv;
use serde::Deserialize;

/// struct holding configuration options
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// port to be used for gRPC server
    pub docker_port_grpc: u16,
    /// port to be used for REST server
    pub docker_port_rest: u16,
    /// storage microservice host
    pub storage_host_grpc: String,
    /// storage microservice port
    pub storage_port_grpc: u16,
    /// path to log configuration YAML file
    pub log_config: String,
    /// Rate limit - requests per second for REST requests
    pub rest_request_limit_per_second: u8,
    /// Enforces a limit on the concurrent number of requests the underlying service can handle
    pub rest_concurrency_limit_per_service: u8,
    /// Full url (including port number) to be allowed as request origin for
    /// REST requests
    pub rest_cors_allowed_origin: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Default values for Config
    pub fn new() -> Self {
        Config {
            docker_port_grpc: 50051,
            docker_port_rest: 8000,
            storage_host_grpc: String::from("svc-storage"),
            storage_port_grpc: 50008,
            log_config: String::from("log4rs.yaml"),
            rest_request_limit_per_second: 2,
            rest_concurrency_limit_per_service: 5,
            rest_cors_allowed_origin: String::from("http://localhost:3000"),
        }
    }

    /// Create a new `Config` object using environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        // read .env file if present
        dotenv().ok();
        let default_config = Config::default();
        config::Config::builder()
            .set_default("docker_port_grpc", 50051)?
            .set_default("docker_port_rest", 8000)?
            .set_default("storage_host_grpc", String::from("svc-storage"))?
            .set_default("storage_port_grpc", 50008)?
            .set_default("log_config", String::from("log4rs.yaml"))?
            .set_default(
                "rest_concurrency_limit_per_service",
                default_config.rest_concurrency_limit_per_service,
            )?
            .set_default(
                "rest_request_limit_per_seconds",
                default_config.rest_request_limit_per_second,
            )?
            .set_default(
                "rest_cors_allowed_origin",
                default_config.rest_cors_allowed_origin,
            )?
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
