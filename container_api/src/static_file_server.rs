use config::{UiBundleConfiguration, UiInterfaceConfiguration, InterfaceConfiguration};
use error::HolochainResult;
use hyper::{
    http::{uri, response::Builder},
    rt::{self, Future},
    server::Server,
    Body, Request, Response
};
use std::{io::Error, thread};
// use tokio::runtime::Runtime;
use hyper_staticfile::{Static, StaticFuture};
use tokio::prelude::{future, Poll, Async};

const DNA_CONFIG_ROUTE: &str = "/_dna_connections.json";


fn redirect_request_to_root<T>(req: &mut Request<T>) {
    let mut original_parts: uri::Parts = req.uri().to_owned().into();
    original_parts.path_and_query = Some("/".parse().unwrap());
    *req.uri_mut() = uri::Uri::from_parts(original_parts).unwrap();
}

fn dna_connections_response(config: &Option<InterfaceConfiguration>) -> Response<Body> {
    let intefaces = match config {
        Some(config) => {
            json!([config])
        },
        None => {
            json!([])
        }
    };
    Builder::new()
        .body(json!({"dna_interfaces": intefaces}).to_string().into())
        .expect("unable to build response")
}


enum MainFuture {
    Static(StaticFuture<Body>),
    Config(Option<InterfaceConfiguration>),
}

impl Future for MainFuture {
    type Item = Response<Body>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            MainFuture::Config(ref config) => {
                Ok(Async::Ready(dna_connections_response(config)))
            },
            MainFuture::Static(ref mut future) => {
                future.poll()
            }
        }
    }
}

/// Hyper `Service` implementation that serves all requests.
struct StaticService {
    static_: Static,
    dna_interface_config: Option<InterfaceConfiguration>,
}

impl StaticService {
    fn new(path: &String, dna_interface_config: &Option<InterfaceConfiguration>) -> Self {
        StaticService {
            static_: Static::new(path),
            dna_interface_config: dna_interface_config.to_owned(),
        }
    }
}

impl hyper::service::Service for StaticService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = MainFuture;

    fn call(&mut self, mut req: Request<Body>) -> MainFuture {
        match req.uri().path() {
            DNA_CONFIG_ROUTE => MainFuture::Config(self.dna_interface_config.clone()),
            _ => {
                MainFuture::Static(
                    hyper_staticfile::resolve(&self.static_.root, &req)
                    .map(|result| {
                        match result {
                            hyper_staticfile::ResolveResult::NotFound => {
                                // redirect all not-found routes to the root
                                // this allows virtual routes on the front end
                                redirect_request_to_root(&mut req);
                                self.static_.serve(req)
                            }
                            _ => self.static_.serve(req),
                        }
                    })
                    .wait()
                    .unwrap()
                )
            }
        }
    }
}

pub struct StaticServer {
    #[allow(dead_code)]
    shutdown_signal: Option<futures::channel::oneshot::Sender<()>>,
    config: UiInterfaceConfiguration,
    #[allow(dead_code)]
    bundle_config: UiBundleConfiguration,
    connected_dna_interface: Option<InterfaceConfiguration>,
    running: bool,
}

impl StaticServer {
    pub fn from_configs(
        config: UiInterfaceConfiguration,
        bundle_config: UiBundleConfiguration,
        connected_dna_interface: Option<InterfaceConfiguration>,
    ) -> Self {
        StaticServer {
            shutdown_signal: None,
            config,
            bundle_config,
            connected_dna_interface,
            running: false,
        }
    }

    pub fn start(&mut self) -> HolochainResult<()> {
        let addr = ([127, 0, 0, 1], self.config.port).into();

        // let (tx, rx) = futures::channel::oneshot::channel::<()>();
        // self.shutdown_signal = Some(tx);
        let static_path = self.bundle_config.root_dir.to_owned();
        let dna_interfaces = self.connected_dna_interface.to_owned();

        println!(
            "About to serve path \"{}\" at http://{}",
            &self.bundle_config.root_dir, &addr
        );
        self.running = true;

        thread::spawn(move || {
            let server = Server::bind(&addr)
                .serve(move || future::ok::<_, Error>(StaticService::new(&static_path, &dna_interfaces)))
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
    use crate::config::InterfaceDriver;

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

        let test_dna_interface = InterfaceConfiguration {
            id: "interface".to_string(),
            admin: true,
            driver: InterfaceDriver::Http{port: 3000},
            instances: Vec::new(),
        };

        let mut static_server = StaticServer::from_configs(test_config, test_bundle_config, Some(test_dna_interface));
        assert_eq!(static_server.start(), Ok(()))
    }
}
