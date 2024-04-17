use config::Config;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    pub env: Env,
    pub storage_path: String,
    pub alias_length: i32,
    pub http_server: HttpServer,
}
impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            env: Env::Local,
            storage_path: String::from("storage.db"),
            alias_length: 6,
            http_server: HttpServer {
                address: String::from("0.0.0.0:8080"),
                timeout: 4,
                idle_timeout: 60,
            },
        }
    }
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Env {
    Local,
    Prod,
    Dev,
}
#[derive(Debug, Clone, Deserialize)]
pub struct HttpServer {
    pub address: String,
    pub timeout: i32,
    pub idle_timeout: i32,
}

pub fn init(file_name: &str) -> Configuration {
    let settings = Config::builder()
        .add_source(config::File::with_name(file_name))
        .build()
        .expect("No config file found");
    settings
        .try_deserialize::<Configuration>()
        .unwrap_or_default()
}
