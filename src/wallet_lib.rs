use secp256k1::{PublicKey, SecretKey};
use anyhow::{Result, Error};
use secp256k1::rand::{rngs, SeedableRng};
use web3::Web3;
use web3::transports::Http;
use web3::types::{TransactionParameters, U256, H256, H160};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use std::env;
use bytes::Bytes;
use std::sync::{Arc, Mutex};
use serde::Serialize;

pub fn create_keypair() -> Result<(SecretKey, PublicKey)> {
    let secp = secp256k1::Secp256k1::new();
    let mut rng = rngs::StdRng::seed_from_u64(6);
    Ok(secp.generate_keypair(&mut rng))
}

pub fn establish_web3_connection(url: &str) -> Result<Web3<Http>> {
    let transport = web3::transports::Http::new(url)?;
    Ok(web3::Web3::new(transport))
}

pub fn create_txn_object(to: H160, value: usize) -> Result<TransactionParameters> {
    Ok(TransactionParameters {
        to: Some(to),
        value: U256::exp10(value), //0.1 eth
        ..Default::default()
    })
}

pub async fn sign_and_send(web3: Web3<Http>, tx_object: TransactionParameters, seckey: SecretKey) -> Result<H256> {
    let signed = web3.accounts().sign_transaction(tx_object, &seckey).await?;
    Ok(web3.eth().send_raw_transaction(signed.raw_transaction).await?)
}

#[derive(Serialize, Clone, Debug)] 
pub struct KeyPair {
    pub public_key: Vec<u8>, 
    pub secret_key: Vec<u8>, 
}

pub async fn send_keypair_to_backup_service(keypair: &KeyPair) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.post("https://backup-service-vixlkkqxkq-uc.a.run.app/add_keypair")
        .json(keypair) //parse keypair as JSON before sending it to Google Cloud bucket
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to send keypair to backup service"))
    }
}
