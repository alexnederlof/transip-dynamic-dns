use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use tokio::task::JoinHandle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenResponse {
    token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DnsEntry {
    name: String,
    expire: u32,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DnsEntries {
    #[serde(rename = "dnsEntries")]
    pub entries: Vec<DnsEntry>,
}

const IP_SOURCES: [&str; 8] = [
    "https://icanhazip.com/",
    "https://myexternalip.com/raw",
    "https://ifconfig.io/ip",
    "https://ipecho.net/plain",
    "https://checkip.amazonaws.com/",
    "https://ident.me/",
    "http://whatismyip.akamai.com/",
    "https://myip.dnsomatic.com/",
    // "https://diagnostic.opendns.com/myip",
];

pub async fn get_ip() -> Result<String, String> {
    let accumulate = Arc::new(Mutex::new(HashMap::<String, u16>::new()));
    let futures: Vec<JoinHandle<Result<String, String>>> = IP_SOURCES
        .iter()
        .map(|source| {
            let accumulate = Arc::clone(&accumulate);
            tokio::spawn(async move {
                match reqwest::get(*source).await {
                    Ok(resp) => match resp.text().await {
                        Ok(ip) => {
                            let ip = ip.trim().to_owned();
                            let mut map = accumulate.lock().unwrap();
                            let count = map.entry(ip.clone()).or_insert(0);
                            *count += 1;
                            info!("Got external IP {} from {}", ip, source);
                            Ok(ip)
                        }
                        Err(e) => Err(format!("Cannot get from {}: {:?}", source, e)),
                    },
                    Err(e) => Err(format!("Cannot get from {}: {:?}", source, e)),
                }
            })
        })
        .collect();
    for f in futures {
        match f.await.unwrap() {
            Ok(_) => {}
            Err(e) => warn!("One IP source failed: {:?}", e),
        }
    }

    let mut count: u16 = 0;
    let mut popular: Option<String> = None;
    for (key, value) in accumulate.lock().unwrap().iter() {
        if value > &count {
            count = *value;
            popular = Some(key.to_string());
        }
    }
    popular.ok_or_else(|| "No IP address was resolved".to_string())
}
