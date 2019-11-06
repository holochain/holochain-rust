extern crate structopt;
extern crate tempfile;
#[macro_use]
extern crate serde_json;

//use log::error;
//use std::process::exit;
use self::tempfile::tempdir;
use jsonrpc_core::{MetaIoHandler, Params, Value};
use jsonrpc_ws_server::{RequestContext, ServerBuilder};
use jsonrpc_pubsub::{PubSubHandler, Session};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use serde_json::map::Map;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, RwLock},
};
use structopt::StructOpt;

pub type RpcHandler = PubSubHandler<Arc<Session>>;

/*type Error = String;
fn exec_output<P, S1, I, S2>(cmd: S1, args: I, dir: P, ignore_errors: bool) -> Result<String, Error>
where
    P: AsRef<std::path::Path>,
    S1: AsRef<std::ffi::OsStr>,
    I: IntoIterator<Item = S2>,
    S2: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(cmd);
    cmd.args(args)
        //        .env("N3H_VERSION_EXIT", "1")
        //        .env("NO_CLEANUP", "1")
        .current_dir(dir);
    let res = cmd
        .output()
        .map_err(|e| format!("Failed to execute {:?}: {:?}", cmd, e))?;
    if !ignore_errors && !res.status.success() {
        panic!(
            "bad exit {:?} {:?}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&res.stdout).trim().to_string())
}*/

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        long,
        short,
        help = "The port to run the trycp server at",
        default_value = "9000"
    )]
    port: u16,
}

struct Store {
    dir: tempfile::TempDir,
}

impl Store {
    pub fn new() -> Self {
        Store {
            dir: tempdir().expect("should create tmp dir"),
        }
    }
    pub fn reset(&mut self) {
        self.dir = tempdir().expect("should create tmp dir")
    }
}

fn unwrap_params_map(params: Params) -> Result<Map<String, Value>, jsonrpc_core::Error> {
    match params {
        Params::Map(map) => Ok(map),
        _ => Err(jsonrpc_core::Error::invalid_params("expected params map")),
    }
}

fn get_as_string<T: Into<String>>(
    key: T,
    params_map: &Map<String, Value>,
) -> Result<String, jsonrpc_core::Error> {
    let key = key.into();
    Ok(params_map
        .get(&key)
        .ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` param not provided", &key))
        })?
        .as_str()
        .ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` is not a valid json string", &key))
        })?
        .to_string())
}
fn get_as_bool<T: Into<String>>(
    key: T,
    params_map: &Map<String, Value>,
    default: Option<bool>,
) -> Result<bool, jsonrpc_core::Error> {
    let key = key.into();
    match params_map.get(&key) {
        Some(value) => value.as_bool().ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` has to be a boolean", &key))
        }),
        None => default.ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("required param `{}` not provided", &key))
        }),
    }
}

const CONDUCTOR_CONFIG_FILE_NAME: &str = "conductor-config.toml";

fn get_dir(temp_path_arc: Arc<RwLock<Store>>, id: &String) -> PathBuf {
    let temp_path = temp_path_arc.read().expect("should_lock");
    temp_path.dir.path().join(id).clone()
}

fn get_file(temp_path_arc: Arc<RwLock<Store>>, id: &String) -> PathBuf {
    get_dir(temp_path_arc, id)
        .join(CONDUCTOR_CONFIG_FILE_NAME)
        .clone()
}

fn main() {
    let args = Cli::from_args();
    let mut io: RpcHandler = PubSubHandler::new(MetaIoHandler::default());

    let temp_path_arc: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store::new()));
    let temp_path_arc_setup = temp_path_arc.clone();
    let temp_path_arc_player = temp_path_arc.clone();
    let temp_path_arc_spawn = temp_path_arc.clone();
    let temp_path_arc_kill = temp_path_arc.clone();

    let players_arc: Arc<RwLock<HashMap<String, Child>>> = Arc::new(RwLock::new(HashMap::new()));
    let players_arc_kill = players_arc.clone();
    let players_arc_reset = players_arc.clone();
    let players_arc_spawn = players_arc.clone();

    io.add_method("ping", |_params: Params| Ok(Value::String("pong".into())));

    io.add_method("reset", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let killall = get_as_bool("killall", &params_map, Some(false))?;
        let mut players = players_arc_reset.write().expect("should_lock");
        if killall {
            let output = Command::new("killall")
                .args(&["holochain", "-s", "SIGKILL"])
                .output()
                .expect("failed to execute process");
            println!("killall result: {:?}", output);
        } else {
            for (id, child) in &*players {
                let _ = do_kill(id, child, "SIGKILL"); //ignore any errors
            }
        }
        players.clear();
        let mut temp_path = temp_path_arc.write().expect("should_lock");
        temp_path.reset();

        Ok(Value::String("reset".into()))
    });

    // Return to try-o-rama information it can use to build config files
    // i.e. ensure ports are open, and ensure that configDir is the same one
    // that the actual config will be written to
    io.add_method("setup", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let file_path = get_dir(temp_path_arc_setup.clone(), &id);
        Ok(json!({
            "adminPort": 1111,
            "zomePort": 2222,
            "configDir": file_path.to_string_lossy(),
        }))
    });

    io.add_method("player", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let config_base64 = get_as_string("config", &params_map)?;
        let content =
            base64::decode(&config_base64).map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                message: format!("error decoding config: {:?}", e),
                data: None,
            })?;
        let dir_path = get_dir(temp_path_arc_player.clone(), &id);
        std::fs::create_dir_all(dir_path.clone()).map_err(|e| {
            jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                message: format!(
                    "error making temporary directory for config: {:?} {:?}",
                    e, dir_path
                ),
                data: None,
            }
        })?;
        let file_path = get_file(temp_path_arc_player.clone(), &id);
        File::create(file_path.clone())
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!(
                    "unable to create config file: {:?} {}",
                    e,
                    file_path.to_string_lossy()
                ),
                data: None,
            })?
            .write_all(&content[..])
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!(
                    "unable to write config file: {:?} {}",
                    e,
                    file_path.to_string_lossy()
                ),
                data: None,
            })?;

        let response = format!(
            "wrote config for player {} to {}",
            id,
            file_path.to_string_lossy()
        );
        println!("player {}: {:?}", id, response);
        Ok(Value::String(response))
    });

    io.add_method("spawn", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;

        check_player_config(temp_path_arc_spawn.clone(), &id)?;
        let mut players = players_arc_spawn.write().expect("should_lock");
        if players.contains_key(&id) {
            return Err(jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                message: format!("{} is already running", id),
                data: None,
            });
        };

        let player_config = format!(
            "{}",
            get_file(temp_path_arc_spawn.clone(), &id).to_string_lossy()
        );
        let mut conductor = Command::new("holochain")
            .stdout(Stdio::piped())
            .args(&["-c", &player_config])
            .spawn()
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to spawn conductor: {:?}", e),
                data: None,
            })?;

        match conductor.stdout.take() {
            Some(stdout) => {
                for line in BufReader::new(stdout).lines() {
                    if line.unwrap() == "Done. All interfaces started." {
                        break;
                    }
                }

                players.insert(id.clone(), conductor);
                let response = format!("conductor spawned for {}", id);
                Ok(Value::String(response))
            }
            None => {
                conductor.kill().unwrap();
                return Err(jsonrpc_core::types::error::Error {
                    code: jsonrpc_core::types::error::ErrorCode::InternalError,
                    message: format!("Conductor process not capturing stdout, bailing!"),
                    data: None,
                });
            }
        }
    });

    io.add_method("kill", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let signal = get_as_string("signal", &params_map)?; // TODO: make optional?

        check_player_config(temp_path_arc_kill.clone(), &id)?;
        let mut players = players_arc_kill.write().unwrap();
        match players.remove(&id) {
            None => {
                return Err(jsonrpc_core::types::error::Error {
                    code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                    message: format!("no conductor spawned for {}", id),
                    data: None,
                });
            }
            Some(ref mut child) => {
                do_kill(&id, child, signal.as_str())?;
            }
        }
        let response = format!("killed conductor for {}", id);
        Ok(Value::String(response))
    });

    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender().clone()))
    })
    .start(&format!("0.0.0.0:{}", args.port).parse().unwrap())
    .expect("server should start");
    println!("waiting for connections on port {}", args.port);

    server.wait().expect("server should wait");
}

fn do_kill(
    id: &String,
    child: &Child,
    signal: &str,
) -> Result<(), jsonrpc_core::types::error::Error> {
    let sig = match signal {
        "SIGKILL" => Signal::SIGKILL,
        "SIGTERM" => Signal::SIGTERM,
        _ => Signal::SIGINT,
    };
    signal::kill(Pid::from_raw(child.id() as i32), sig).map_err(|e| {
        jsonrpc_core::types::error::Error {
            code: jsonrpc_core::types::error::ErrorCode::InternalError,
            message: format!("unable to run kill conductor for {} script: {:?}", id, e),
            data: None,
        }
    })
}

fn check_player_config(
    temp_path_arc: Arc<RwLock<Store>>,
    id: &String,
) -> Result<(), jsonrpc_core::types::error::Error> {
    let file_path = get_file(temp_path_arc, id);
    if !file_path.is_file() {
        return Err(jsonrpc_core::types::error::Error {
            code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
            message: format!("player config for {} not setup", id),
            data: None,
        });
    }
    Ok(())
}
