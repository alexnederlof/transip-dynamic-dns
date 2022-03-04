extern crate chrono;
extern crate chrono_tz;
extern crate hmac;
extern crate jwt;
extern crate openssl;

extern crate serde_json;
extern crate sha2;
extern crate simplelog;
use dotenv::dotenv;

use openssl::pkey::PKey;
use simple_error::{SimpleError, SimpleResult};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::env;
use std::fs::File;
use std::io::Read;
mod get_ip;
mod transip;

pub struct AppConfig {
    domain_prefix: String,
    domain: String,
    login: String,
    key: PKey<openssl::pkey::Private>,
}

impl AppConfig {
    fn new() -> SimpleResult<Self> {
        let mut buffer = Vec::new();
        let mut f = File::open(get_env_or_error("TRANSIP_KEY_PATH")?)
            .map_err(|e| SimpleError::new(format!("Cannot read file {:?}", e)))?;
        f.read_to_end(&mut buffer)
            .map_err(|e| SimpleError::new(format!("Cannot read file {:?}", e)))?;
        let key = PKey::private_key_from_pem(&buffer)
            .map_err(|e| SimpleError::new(format!("Not a valid key file {:?}", e)))?;
        Ok(AppConfig {
            domain_prefix: get_env_or_error("TRANSIP_PREFIX")?,
            domain: get_env_or_error("TRANSIP_DOMAIN")?,
            login: get_env_or_error("TRANSIP_LOGIN")?,
            key,
        })
    }
}

fn get_env_or_error(name: &str) -> SimpleResult<String> {
    let result = env::var(name)
        .map_err(|e| format!("Cannot get env var {:?}", e))
        .map_err(SimpleError::new)?;
    if result.trim().is_empty() {
        return SimpleResult::Err(SimpleError::new(format!("{:?} not set", name)));
    }
    SimpleResult::Ok(String::from(result.trim()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Updating IP");
    dotenv().ok();
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("Cannot setup logging?");

    let config = AppConfig::new().unwrap();
    let api = transip::TransIp::new(config);
    api.update_ip_for_address().await?;
    Ok(())
}
