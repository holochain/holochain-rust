use config::{UiBundleConfiguration, UiInterfaceConfiguration};
use error::HolochainResult;
use hyper::{
    rt::{self, Future},
    server::Server,
    Body, Request,
    http::uri,
};
use std::{io::Error, thread};
// use tokio::runtime::Runtime;
use hyper_staticfile::{Static, StaticFuture};
use tokio::prelude::future;

fn redirect_request_to_root<T>(req: &mut Request<T>) {
    let mut original_parts: uri::Parts = req.uri().to_owned().into();
    original_parts.path_and_query = Some("/".parse().unwrap());
    *req.uri_mut() = uri::Uri::from_parts(original_parts).unwrap();
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

    fn call(&mut self, mut req: Request<Body>) -> StaticFuture<Body> {
        hyper_staticfile::resolve(&self.static_.root, &req).map(|result| {
            match result {
                hyper_staticfile::ResolveResult::NotFound => {
                    // redirect all not-found routes to the root
                    // this allows virtual routes on the front end
                    redirect_request_to_root(&mut req);
                    self.static_.serve(req)                    
                },
                _ => self.static_.serve(req)
            }
        }).wait().unwrap()
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
            dna_interface: Some("interface".to_string()),
        };

        let mut static_server = StaticServer::from_configs(test_bundle_config, test_config);
        assert_eq!(static_server.start(), Ok(()))
    }
}
