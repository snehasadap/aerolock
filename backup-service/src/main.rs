use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use secp256k1::{PublicKey, SecretKey};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use bytes::Bytes;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use std::env;
use log::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    //Google Cloud project credentials
    env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/sneha/Downloads/lexical-period-401317-ea4d8a7c28cd.json");

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);

    let keypairs: Arc<Mutex<Vec<KeyPair>>> = Arc::new(Mutex::new(Vec::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(keypairs.clone()))
            .route("/backup", web::get().to(backup))
            .route("/add_keypair", web::post().to(add_keypair))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}

async fn backup(
    client: web::Data<Client>,
    keypairs: web::Data<Arc<Mutex<Vec<KeyPair>>>>,
) -> impl Responder {
    match backup_keys_to_gcs(&client, &keypairs).await {
        Ok(_) => HttpResponse::Ok().body("Backup Successful"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Backup Failed: {}", e)),
    }
}

async fn add_keypair(
    keypair: web::Json<KeyPair>,
    keypairs: web::Data<Arc<Mutex<Vec<KeyPair>>>>,
) -> impl Responder {
    let mut keypairs = keypairs.lock().unwrap();
    keypairs.push(keypair.into_inner());
    HttpResponse::Ok().body("KeyPair Added")
}

async fn backup_keys_to_gcs(
    client: &Client,
    keypairs: &Arc<Mutex<Vec<KeyPair>>>,
) -> Result<()> {   //transfer keys to Google Cloud bucket
    let bucket = "key-backup"; //your bucket name
    let keypairs = keypairs.lock().unwrap();
    for (i, keypair) in keypairs.iter().enumerate() {
        let data = format!("{:?}", keypair);
        //Keypair logging
        info!("Processing keypair {}: {:?}", i, keypair); 
        let object_name = format!("backup-keys-{}", i);
        let upload_request = UploadObjectRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };
        let bytes = Bytes::from(data);
        let media = Media::new(object_name.clone());

        client
            .upload_object(&upload_request, bytes, &UploadType::Simple(media))
            .await?;
    }
    Ok(())
}
