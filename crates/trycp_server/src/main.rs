extern crate structopt;
extern crate tempfile;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

//use log::error;
//use std::process::exit;
use self::tempfile::tempdir;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_ws_server::ServerBuilder;
use serde_json::map::Map;
use std::{fs::File, io::Write, process::Command, path::PathBuf};
use structopt::StructOpt;

type Error = String;
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
}
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

const CONDUCTOR_CONFIG_FILE_NAME :&str = "conductor-config.toml";
lazy_static! {
    pub static ref TEMP_PATH: tempfile::TempDir = tempdir().expect("should create tmp dir");
}

fn main() {
    let args = Cli::from_args();
    let mut io = IoHandler::new();

    fn get_dir(id: &String) -> PathBuf {
        TEMP_PATH.path().join(id).clone()
    }

    fn get_file(id: &String) -> PathBuf {
        get_dir(id).join(CONDUCTOR_CONFIG_FILE_NAME).clone()
    }

    io.add_method("ping", |_params: Params| Ok(Value::String("pong".into())));

    // Return to try-o-rama information it can use to build config files
    // i.e. ensure ports are open, and ensure that configDir is the same one
    // that the actual config will be written to
    io.add_method("setup", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let file_path = get_dir(&id);
        Ok(json!({
            "adminPort": 1111,
            "zomePort": 2222,
            "configDir": file_path.to_string_lossy(),
        }))
    });

    io.add_method("player", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let config_base64 = get_as_string("config", &params_map)?;
        let content =
            base64::decode(&config_base64).map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                message: format!("error decoding config: {:?}", e),
                data: None,
            })?;
        let dir_path = get_dir(&id);
        std::fs::create_dir_all(dir_path.clone()).map_err(|e| jsonrpc_core::types::error::Error {
            code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
            message: format!("error making temporary directory for config: {:?} {:?}", e, dir_path),
            data: None,
        })?;
        let file_path = get_file(&id);
        File::create(file_path.clone())
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to create config file: {:?} {:?}", e, file_path),
                data: None,
            })?
            .write_all(&content[..])
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to write config file: {:?} {:?}", e, file_path),
                data: None,
            })?;
        println!("Wrote config for player {} to {:?}", id, file_path);
        let response = exec_output(
            "bash",
            vec!["hcm.bash", "player", &id, &file_path.to_string_lossy()],
            ".",
            true,
        )
        .map_err(|e| jsonrpc_core::types::error::Error {
            code: jsonrpc_core::types::error::ErrorCode::InternalError,
            message: format!("unable to run hcm script: {:?}", e),
            data: None,
        })?;
        println!("player {}: {:?}", id, response);
        Ok(Value::String(response))
    });
    io.add_method("spawn", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let response =
            exec_output("bash", vec!["hcm.bash", "spawn", &id], ".", true).map_err(|e| {
                jsonrpc_core::types::error::Error {
                    code: jsonrpc_core::types::error::ErrorCode::InternalError,
                    message: format!("unable to run hcm script: {:?}", e),
                    data: None,
                }
            })?;
        println!("spawn {}: {:?}", id, response);
        Ok(Value::String(response))
    });
    io.add_method("kill", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let signal = get_as_string("signal", &params_map)?; // TODO: make optional?
        let response = exec_output("bash", vec!["hcm.bash", "kill", &id, &signal], ".", true)
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to run hcm script: {:?}", e),
                data: None,
            })?;
        println!("kill {}: {:?}", id, response);
        Ok(Value::String(response))
    });

    let server = ServerBuilder::new(io)
        .start(&format!("0.0.0.0:{}", args.port).parse().unwrap())
        .expect("server should start");
    println!("waiting for connections on port {}", args.port);

    server.wait().expect("server should wait");
}
