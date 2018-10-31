fn load(toml: &str) {
    let agents = load_configuration::<Vec<AgentConfiguration>>(toml);
    let DNAs = load_configuration::<Vec<DNAConfiguration>>(toml);
    let instnaces = load_configuration::<Vec<InstanceConfiguration>>(toml);
    let bridges = load_configuration::<Vec<Bridges>>(toml);
}
