use holochain_net::{p2p_config::*, tweetlog::*};

/// Create a P2pConfig for an IPC node that uses n3h and possibily a specific folder.
/// Return the generated P2pConfig and the created tempdir if no dir was provided.
#[cfg_attr(tarpaulin, skip)]
pub(crate) fn create_ipc_config(
    maybe_config_filepath: Option<&str>,
    maybe_end_user_config_filepath: Option<String>,
    bootstrap_nodes: Vec<String>,
    maybe_dir_path: Option<String>,
) -> (P2pConfig, Option<tempfile::TempDir>) {
    // Create temp directory if no dir was provided
    let mut maybe_dir_ref = None;
    let dir = if let Some(dir_path) = maybe_dir_path {
        dir_path
    } else {
        let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
        let dir_path = dir_ref.path().clone().to_string_lossy().to_string();
        maybe_dir_ref = Some(dir_ref);
        dir_path
    };

    log_i!("create_ipc_config() dir = {}", dir);

    // Create config
    let mut config: P2pConfig = match maybe_config_filepath {
        Some(filepath) => {
            log_d!("filepath = {}", filepath);
            // Get config from file
            let p2p_config = P2pConfig::from_file(filepath);
            assert_eq!(p2p_config.backend_kind, P2pBackendKind::IPC);
            // complement missing fields
            serde_json::from_value(json!({
            "backend_kind": String::from(p2p_config.backend_kind),
            "backend_config":
            {
                "socketType": p2p_config.backend_config["socketType"],
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": p2p_config.backend_config["spawn"]["env"]["N3H_MODE"],
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": p2p_config.backend_config["spawn"]["env"]["N3H_IPC_SOCKET"],
                        "N3H_LOG_LEVEL": p2p_config.backend_config["spawn"]["env"]["N3H_LOG_LEVEL"],
                    }
                },
            }})).expect("Failled making valid P2pConfig with filepath")
        }
        None => {
            // use default config
            serde_json::from_value(json!({
            "backend_kind": "IPC",
            "backend_config":
            {
                "socketType": "ws",
                "bootstrapNodes": bootstrap_nodes,
                "spawn":
                {
                    "workDir": dir.clone(),
                    "env": {
                        "N3H_MODE": "HACK",
                        "N3H_WORK_DIR": dir.clone(),
                        "N3H_IPC_SOCKET": "tcp://127.0.0.1:*",
                        "N3H_LOG_LEVEL": "t"
                }
            },
            }}))
            .expect("Failed making valid default P2pConfig")
        }
    };
    config.maybe_end_user_config = Some(P2pConfig::load_end_user_config(
        maybe_end_user_config_filepath,
    ));
    return (config, maybe_dir_ref);
}

/// Create a P2pConfig for a node that uses LIB3H and possibily a specific persistance folder.
/// Return the generated P2pConfig and the created tempdir if no dir was provided.
#[cfg_attr(tarpaulin, skip)]
pub(crate) fn create_lib3h_config(
    maybe_config_filepath: Option<&str>,
    maybe_end_user_config_filepath: Option<String>,
    bootstrap_nodes: Vec<String>,
    maybe_dir_path: Option<String>,
) -> (P2pConfig, Option<tempfile::TempDir>) {
    // Create temp directory if no dir was provided
    let mut maybe_dir_ref = None;
    let dir = if let Some(dir_path) = maybe_dir_path {
        dir_path
    } else {
        let dir_ref = tempfile::tempdir().expect("Failed to created a temp directory.");
        let dir_path = dir_ref.path().clone().to_string_lossy().to_string();
        maybe_dir_ref = Some(dir_ref);
        dir_path
    };

    log_i!("create_lib3h_config() dir = {}", dir);

    // Create config
    let mut config: P2pConfig = match maybe_config_filepath {
        Some(filepath) => {
            log_d!("filepath = {}", filepath);
            // Get config from file
            let p2p_config = P2pConfig::from_file(filepath);
            assert_eq!(p2p_config.backend_kind, P2pBackendKind::LIB3H);
            // complement missing fields
            serde_json::from_value(json!({
            "backend_kind": "LIB3H",
            "backend_config":
            {
                "socketType": p2p_config.backend_config["socketType"],
                "bootstrapNodes": bootstrap_nodes,
                "workDir": dir.clone(),
                "logLevel": p2p_config.backend_config["logLevel"],
            }}))
            .expect("Failled making valid LIB3H P2pConfig")
        }
        None => {
            // use default config
            serde_json::from_value(json!({
            "backend_kind": "LIB3H",
            "backend_config":
            {
                "socketType": "ws",
                "bootstrapNodes": bootstrap_nodes,
                "workDir": dir.clone(),
                "logLevel": "t"
            },
            }))
            .expect("Failed making Lib3h default P2pConfig")
        }
    };
    config.maybe_end_user_config = Some(P2pConfig::load_end_user_config(
        maybe_end_user_config_filepath,
    ));
    return (config, maybe_dir_ref);
}
