use static_file_server::{
    // dna_connections_response,
    // redirect_request_to_root,
    ConductorStaticFileServer,
    // DNA_CONFIG_ROUTE,
};
use config::{InterfaceConfiguration, UiBundleConfiguration, UiInterfaceConfiguration};
use error::HolochainResult;
use holochain_core_types::error::HolochainError;

use std::{
    sync::mpsc::{Sender},
};

pub struct NickelStaticServer {
    shutdown_signal: Option<Sender<()>>,
    config: UiInterfaceConfiguration,
    bundle_config: UiBundleConfiguration,
    connected_dna_interface: Option<InterfaceConfiguration>,
    running: bool,
}

impl ConductorStaticFileServer for NickelStaticServer {

    fn from_configs(
        config: UiInterfaceConfiguration,
        bundle_config: UiBundleConfiguration,
        connected_dna_interface: Option<InterfaceConfiguration>,
    ) -> Self {
        Self {
            shutdown_signal: None,
            config,
            bundle_config,
            connected_dna_interface,
            running: false,
        }
    }

    fn start(&mut self) -> HolochainResult<()> {
        Err(HolochainError::ErrorGeneric("Not Implemented".into()).into())
    }

    fn stop(&mut self) -> HolochainResult<()> {
        match self.shutdown_signal.clone() {
            Some(_shutdown_signal) => {
                Err(HolochainError::ErrorGeneric("Not Implemented".into()).into())
            }
            None => Err(HolochainError::ErrorGeneric("server is already stopped".into()).into()),
        }
    }
}