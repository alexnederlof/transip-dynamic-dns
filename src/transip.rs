use crate::AppConfig;
use chrono::prelude::*;
use log::info;
use openssl::base64::encode_block;
use openssl::hash::MessageDigest;

use crate::get_ip::get_ip;
use openssl::sign::Signer;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

pub struct TransIp {
    config: AppConfig,
}
impl TransIp {
    pub fn new(config: AppConfig) -> Self {
        TransIp { config }
    }

    pub async fn update_ip_for_address(&self) -> Result<(), String> {
        let client = Client::new();
        let ip = get_ip().await?;
        let token = self.get_token(&client).await?;
        let current = self.get_current_records(&token, &client).await?;
        let mine = current
            .entries
            .into_iter()
            .find(|item| item.name == self.config.domain_prefix)
            .ok_or_else(|| String::from("My IP is not listed"))?;
        if mine.content != ip {
            info!(
                "External IP {:?} does not match record {:?}",
                ip, mine.content
            );
            let new_record = DnsEntry {
                content: ip,
                ..mine
            };
            self.patch_record(&token, &client, &new_record).await?;
        } else {
            info!("External IP {:?} matches record {:?}", ip, mine.content);
        }
        Ok(())
    }
    pub async fn patch_record(
        &self,
        token: &str,
        client: &Client,
        entry: &DnsEntry,
    ) -> Result<(), String> {
        let patch = json!({ "dnsEntry": &entry });
        info!("Patching record to {:?}", patch);
        let resp = client
            .patch(format!(
                "https://api.transip.nl/v6/domains/{}/dns",
                self.config.domain
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&patch)
            .send()
            .await
            .map_err(|e| format!("Could not retrieve token {:?}", e))?;
        if resp.status().is_success() {
            info!("Record updated!");
            Ok(())
        } else {
            Err(format!("Could not update DNS Record: {:?}", resp))
        }
    }
    pub async fn get_current_records(
        &self,
        token: &str,
        client: &Client,
    ) -> Result<DnsEntries, String> {
        info!("Getting current records for {}", self.config.domain);
        let resp = client
            .get(format!(
                "https://api.transip.nl/v6/domains/{}/dns",
                self.config.domain
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Could not retrieve token {:?}", e))?;
        if resp.status().is_success() {
            resp.json::<DnsEntries>()
                .await
                .map_err(|e| format!("Cannot parse JSON {:?}", e))
        } else {
            Err(format!("Got {:?} {:?}", resp.status(), resp.text().await))
        }
    }
    pub async fn get_token(&self, client: &Client) -> Result<String, String> {
        let date: String = Local::now().to_rfc3339().chars().skip(6).take(32).collect();
        let body = json!({
            "login": self.config.login,
            "nonce": &date,
            "read_only": false,
            "expiration_time": "1 minutes",
            "label": format!("Dyn DNS handler @ {}", &date),
            "global_key": true
        });
        info!("Sending token request {:?}", body);
        let signature = self.sign(body.to_string())?;
        let res = client
            .post("https://api.transip.nl/v6/auth")
            .header("Signature", signature)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("Could not retrieve token {:?}", e))?;
        if res.status().is_success() {
            let resp = res
                .json::<TokenResponse>()
                .await
                .map_err(|e| format!("Could not parse json {:?}", e))?;
            Ok(resp.token)
        } else {
            let status = res.status();
            let body = res.text().await;
            Err(format!("Failed to resolve: {:?} {:?}", status, body))
        }
    }
    pub fn sign(&self, content: String) -> Result<String, String> {
        let mut signer = Signer::new(MessageDigest::sha512(), &self.config.key)
            .map_err(|e| format!("Cannot create signer {:?}", e))?;
        signer
            .update(content.as_bytes())
            .map_err(|e| format!("Could not sign {:?}", e))?;
        let result = signer
            .sign_to_vec()
            .map_err(|e| format!("Could not sign {:?}", e))?;
        let result = encode_block(&result);
        Ok(result)
    }
}
