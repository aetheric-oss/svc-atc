//! Re-export of used objects

pub use super::client as atc;
pub use super::service::Client as AtcServiceClient;
pub use atc::AtcClient;

pub use lib_common::grpc::Client;
