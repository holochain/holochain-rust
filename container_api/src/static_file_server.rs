use config::{UiBundleConfiguration, UiInterfaceConfiguration};
use error::HolochainResult;
use hyper::{
    rt::{self, Future},
    server::Server,
    Body, Request,
};
use std::{io::Error, thread};
// use tokio::runtime::Runtime;
use hyper_staticfile::{Static, StaticFuture};
use tokio::prelude::future;

pub fn notify(msg: String) {
    println!("{}", msg);
}

/// Hyper `Service` implementation that serves all requests.
struct StaticService {
    static_: Static,
}

impl StaticService {
    fn new(path: &String) -> Self {
        StaticService {
            static_: Static::new(path),
        }
    }
}

impl hyper::service::Service for StaticService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = StaticFuture<Body>;

    fn call(&mut self, req: Request<Body>) -> StaticFuture<Body> {
        self.static_.serve(req)
    }
}

pub struct StaticServer {
    #[allow(dead_code)]
    shutdown_signal: Option<futures::channel::oneshot::Sender<()>>,
    config: UiInterfaceConfiguration,
    #[allow(dead_code)]
    bundle_config: UiBundleConfiguration,
    running: bool,
}

impl StaticServer {
    pub fn from_configs(
        bundle_config: UiBundleConfiguration,
        config: UiInterfaceConfiguration,
    ) -> Self {
        StaticServer {
            shutdown_signal: None,
            config,
            bundle_config,
            running: false,
        }
    }

    pub fn start(&mut self) -> HolochainResult<()> {
        let addr = ([127, 0, 0, 1], self.config.port).into();

        // let (tx, rx) = futures::channel::oneshot::channel::<()>();
        // self.shutdown_signal = Some(tx);
        let static_path = self.bundle_config.root_dir.to_owned();

        println!(
            "About to serve path \"{}\" at http://{}",
            &self.bundle_config.root_dir, &addr
        );
        self.running = true;

        thread::spawn(move || {
            let server = Server::bind(&addr)
                .serve(move || future::ok::<_, Error>(StaticService::new(&static_path)))
                // .with_graceful_shutdown(rx)
                .map_err(|e| eprintln!("server error: {}", e));

            println!("Listening on http://{}", addr);
            rt::run(server)
        });
        Ok(())
    }

    // pub fn stop(&mut self) -> HolochainResult<()> {
    // 	if let Some(shutdown_signal) = self.shutdown_signal {
    // 		shutdown_signal.send(())
    // 		.and_then(|_| {
    // 			self.running = false;
    // 			self.shutdown_signal = None;
    // 			Ok(())
    // 		});
    // 	}
    // 	Err(HolochainError::ErrorGeneric("server is already stopped".into()).into())
    // }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_build_server() {
        let test_bundle_config = UiBundleConfiguration {
            id: "bundle id".to_string(),
            root_dir: "".to_string(),
            hash: "Qmsdasdasd".to_string(),
        };

        let test_config = UiInterfaceConfiguration {
            id: "an id".to_string(),
            bundle: "a bundle".to_string(),
            port: 3000,
            dna_interface: "interface".to_string(),
        };

        let mut static_server = StaticServer::from_configs(test_bundle_config, test_config);
        assert_eq!(static_server.start(), Ok(()))
    }
}
