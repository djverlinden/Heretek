use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub heretekd: HostPort,
    pub heretekctl: HostPort,
    #[serde(default)]
    pub proxmox: ProxmoxConfig,
}

#[derive(Debug, Deserialize)]
pub struct HostPort {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProxmoxConfig {
    pub version: Option<String>,
}

pub fn load_config() -> AppConfig {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    config.try_deserialize::<AppConfig>().unwrap()
}
