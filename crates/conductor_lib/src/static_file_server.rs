use crate::{
    config::{InterfaceConfiguration, UiBundleConfiguration, UiInterfaceConfiguration},
    error::HolochainResult,
};
use hyper::{http::uri, Request};

pub const DNA_CONFIG_ROUTE: &str = "/_dna_connections.json";

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn redirect_request_to_root<T>(req: &mut Request<T>) {
    let mut original_parts: uri::Parts = req.uri().to_owned().into();
    original_parts.path_and_query = Some("/".parse().unwrap());
    *req.uri_mut() = uri::Uri::from_parts(original_parts).unwrap();
}

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
pub fn dna_connections_response(config: &Option<InterfaceConfiguration>) -> serde_json::Value {
    let interface = match config {
        Some(config) => json!(config),
        None => serde_json::Value::Null,
    };
    json!({ "dna_interface": interface })
}

pub trait ConductorStaticFileServer {
    fn from_configs(
        config: UiInterfaceConfiguration,
        bundle_config: UiBundleConfiguration,
        connected_dna_interface: Option<InterfaceConfiguration>,
    ) -> Self;
    fn start(&mut self) -> HolochainResult<()>;
    fn stop(&mut self) -> HolochainResult<()>;
}
