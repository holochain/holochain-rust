use boolinator::Boolinator;
use crate::config::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct HappBundle {
    pub instances: Vec<HappBundleInstance>,
    pub bridges: Vec<Bridge>,
    #[serde(rename="UIs")]
    pub uis: Vec<HappBundleUi>,
}

#[derive(Serialize, Deserialize)]
pub struct HappBundleInstance {
    pub name: String,
    pub id: String,
    pub dna_hash: String,
    pub uri: String,
    pub dna_properties: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HappBundleUi {
    pub name: String,
    pub id: Option<String>,
    pub uri: String,
    pub instance_references: Vec<HappBundleInstanceReference>,
}

impl HappBundleUi {
    pub fn id(&self) -> String {
        self.id.clone().unwrap_or_else(|| String::from(""))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HappBundleInstanceReference {
    pub ui_handle: String,
    pub instance_id: String,
}

impl HappBundle {
    pub fn id_references_are_consistent(&self) -> Result<(), String> {
        for bridge in self.bridges.iter() {
            for id in vec![bridge.callee_id.clone(), bridge.caller_id.clone()] {
                self.instances.iter().find(|i| i.id == id).ok_or(format!(
                    "No instance with ID {} referenced in bridge {:?}",
                    id, bridge
                ))?;
            }
        }

        for ui in self.uis.iter() {
            for reference in ui.instance_references.iter() {
                self.instances
                    .iter()
                    .find(|i| i.id == reference.instance_id)
                    .ok_or(format!(
                        "No instance with ID {} referenced in UI {:?}",
                        reference.instance_id, ui
                    ))?;
            }
        }
        Ok(())
    }

    pub fn only_file_uris(&self) -> Result<(), String> {
        for instance in self.instances.iter() {
            instance.uri.starts_with("file:").ok_or(format!(
                "Instance {} uses non-file URI which is not supported in `hc run`",
                instance.id
            ))?;
        }

        for ui in self.uis.iter() {
            ui.uri.starts_with("dir:").ok_or(format!(
                "UI {} uses non-dir URI which is not supported in `hc run`",
                ui.id()
            ))?;
        }

        Ok(())
    }

    pub fn build_conductor_config(
        &self,
        ui_port: u16,
        agent_config: AgentConfiguration,
        network: Option<NetworkConfig>,
        logger: LoggerConfiguration,
    ) -> Result<Configuration, String> {
        self.id_references_are_consistent()?;
        self.only_file_uris()?;

        let dnas = self
            .instances
            .iter()
            .map(|happ_instance| {
                // splitting off "file://"
                let file = happ_instance.uri.clone().split_off(5);
                DnaConfiguration {
                    id: happ_instance.id.clone(),
                    file,
                    hash: happ_instance.dna_hash.clone(),
                    uuid: None,
                }
            })
            .collect::<Vec<_>>();

        let instances = self
            .instances
            .iter()
            .map(|happ_instance| InstanceConfiguration {
                id: happ_instance.id.clone(),
                dna: happ_instance.id.clone(),
                agent: agent_config.id.clone(),
                storage: StorageConfiguration::Memory,
            })
            .collect::<Vec<_>>();

        let mut interfaces = Vec::new();
        let mut ui_bundles = Vec::new();
        let mut ui_interfaces = Vec::new();
        for ui in self.uis.iter() {
            interfaces.push(InterfaceConfiguration {
                id: ui.id(),
                driver: InterfaceDriver::Websocket { port: 8000 },
                admin: false,
                instances: ui
                    .instance_references
                    .iter()
                    .map(|ui_ref| InstanceReferenceConfiguration {
                        id: ui_ref.instance_id.clone(),
                        alias: Some(ui_ref.ui_handle.clone()),
                    })
                    .collect(),
            });

            ui_bundles.push(UiBundleConfiguration {
                id: ui.id(),
                root_dir: ui.uri.clone().split_off(4), // splitting off "dir://"
                hash: None,
            });

            ui_interfaces.push(UiInterfaceConfiguration {
                id: ui.id(),
                bundle: ui.id(),
                port: ui_port,
                dna_interface: Some(ui.id()),
                reroute_to_root: false,
                bind_address: String::from("127.0.0.1"),
            });
        }

        Ok(Configuration {
            agents: vec![agent_config],
            dnas,
            instances,
            bridges: self.bridges.clone(),
            interfaces,
            ui_bundles,
            ui_interfaces,
            network,
            logger,
            ..Default::default()
        })
    }
}