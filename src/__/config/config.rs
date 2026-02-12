use super::Log;
// TODO check all crate::* -> super::* or DONT
use crate::*;

// TODO 删除RwLock 而直接用实际的初始化, 或者比如OnceLock<Option>啥的
pub static CONFIG: LazyLock<std::sync::RwLock<Root>> =
    LazyLock::new(|| std::sync::RwLock::new(Root { ..default!() }));

pub fn init_config(root: Root) -> Result<()> {
    let mut config = CONFIG.write()?;
    *config = root;
    Ok(())
}

pub fn is_production() -> bool {
    if cfg!(debug_assertions) {
        return false;
    }
    if let Ok(config) = CONFIG.read() {
        return config.environment == Environment::Production;
    }
    return true;
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Root {
    #[serde(default)]
    pub environment: Environment,
    #[serde(default)]
    pub private_key_hex: String,
    #[serde(default)]
    pub public_key_hex: String,
    #[serde(default)]
    pub log: Log,
    #[cfg(feature = "wasm")]
    #[serde(default)]
    pub wasm: Wasm,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Wasm {
    pub public_key_hexes: Vec<String>,
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Production
    }
}

#[macro_export]
macro_rules! config {
    () => {{
        let root: std::sync::RwLockReadGuard<config::Root> = config::CONFIG.read()?;
        root
    }};
}

#[macro_export]
macro_rules! init_config {
    ($e:expr) => {
        config::init_config($e)?
    };
}

tests! {
    #[cfg(feature = "wasm")]
    fn test_wasm_config() {
        init_config!(Root { wasm: Wasm { public_key_hexes: vec!["04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207".to_string()] } , ..default!() });
        assert_eq!(
            config!().wasm.public_key_hexes,
            vec![
                "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207"
            ]
        );
        // reset config for other test
        init_config!(Root { ..default!() })
    }

    #[cfg(feature = "async")]
    async fn test_config_from_sample() {
        let toml_str = fs::read_to_string("src/__/config/config.toml.sample").await?;
        let config: Root = toml::from_str(&toml_str)?;

        init_config!(config.clone());

        assert_eq!(config.environment, Environment::Development);
        assert!(!is_production());

        assert_eq!(
            config.private_key_hex,
            "b9ab0b828ff68872f21a837fc303668428dea11dcd1b24429d0c99e24eed83d5"
        );
        assert_eq!(
            config!().private_key_hex,
            "b9ab0b828ff68872f21a837fc303668428dea11dcd1b24429d0c99e24eed83d5"
        );

        assert_eq!(
            config.public_key_hex,
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207"
        );
        assert_eq!(
            config!().public_key_hex,
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207"
        );

        assert_eq!(config.log.file_name, "./logs/api.log");
        assert_eq!(config.log.level, log::Level::INFO);
        assert_eq!(config!().log.file_name, "./logs/api.log");
        assert_eq!(config!().log.level, log::Level::INFO);

        // reset config for other test
        init_config!(Root { ..default!() })
    }
}
