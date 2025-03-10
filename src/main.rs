use secp256k1::SecretKey;
use web3;
use tokio;
use web3::types::{Address};
use std::str::FromStr;
use crate::wallet_lib::{create_txn_object, sign_and_send, send_keypair_to_backup_service, KeyPair}; 
use anyhow::{Result};
use fltk::{app, button::Button, enums::{CallbackTrigger, Color, Font, FrameType}, frame::Frame, input, prelude::*, window::Window};
use log::info;

mod wallet_lib;

const URL: &str = "https://eth-ropsten.alchemyapi.io/v2/owrGyNISGOXRtlnncV2XGYZ4a6DvdYi_";

#[derive(Clone, Debug)]
pub enum WalletMessage {
    NewWallet,
    Send,
    Backup, 
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init(); 
    let app = app::App::default();
    let mut wind = Window::default().with_size(500, 800).with_label("Simple Wallet");
    let mut but = Button::new(195, 450, 120, 45, "Create Wallet");
    let mut but2 = Button::new(200, 300, 100, 35, "SEND");
    let mut but3 = Button::new(200, 500, 100, 35, "BACKUP"); 
    let mut inp1 = input::Input::new(200, 200, 225, 35, "To: ");
    let mut frame = Frame::default()
        .with_size(600, 100)
        .center_of(&wind)
        .with_label("0 Wallets");

    frame.set_label_color(Color::White);
    frame.set_label_font(Font::TimesBold);
    frame.set_label_size(24);

    wind.set_color(Color::DarkCyan);

    but.set_color(Color::White);
    but.set_label_color(Color::DarkMagenta);
    but.set_label_font(Font::TimesBold);
    but.set_frame(FrameType::FlatBox);
    but.clear_visible_focus();
    
    but2.set_color(Color::White);
    but2.set_label_color(Color::DarkMagenta);
    but2.set_label_font(Font::TimesBold);
    but2.set_frame(FrameType::FlatBox);
    but2.clear_visible_focus();
  
    but3.set_color(Color::White);
    but3.set_label_color(Color::DarkMagenta);
    but3.set_label_font(Font::TimesBold);
    but3.set_frame(FrameType::FlatBox);
    but3.clear_visible_focus();

    inp1.set_frame(FrameType::FlatBox);

    wind.end();
    wind.show();

    inp1.set_value("Paste Address Here");
    inp1.set_trigger(CallbackTrigger::Changed);

    let (s, r) = app::channel::<WalletMessage>();

    but.emit(s.clone(), WalletMessage::NewWallet);
    but2.emit(s.clone(), WalletMessage::Send);
    but3.emit(s, WalletMessage::Backup); 

    let web3 = wallet_lib::establish_web3_connection(URL)?;
    let mut keypairs: Vec<wallet_lib::KeyPair> = Vec::new();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                WalletMessage::NewWallet => {
                    let (seckey, pubkey) = match wallet_lib::create_keypair() {
                        Ok(val) => val,
                        Err(_e) => unimplemented!(),
                    };
                    let keypair = wallet_lib::KeyPair {
                        public_key: pubkey.serialize().to_vec(),
                        secret_key: seckey[..].to_vec(),
                    };
                    keypairs.push(keypair.clone()); 
                    frame.set_label(&format!("{} Wallets", keypairs.len()));
                    println!("keypairs {:?}", keypairs);
                    info!("Generated new keypair: {:?}", keypair); 
                },
                WalletMessage::Send => {
                    let to_adrs = Address::from_str(&inp1.value().as_str())?;
                    let tx_object = create_txn_object(to_adrs, 7)?;
                    let seckey = SecretKey::from_slice(&keypairs[0].secret_key).unwrap();
                    let result = sign_and_send(web3.clone(), tx_object, seckey).await;
                    match result {
                        Ok(val) => frame.set_label(&format!("{}", val)),
                        Err(e) => frame.set_label(&format!("{}", e)),
                    };
                },
                WalletMessage::Backup => {
                    for keypair in &keypairs {
                        if let Err(e) = send_keypair_to_backup_service(keypair).await {
                            println!("Failed to send keypair to backup service: {}", e);
                            info!("Failed to send keypair to backup service: {}", e); 
                        }
                    }
                    frame.set_label("Backup Triggered Successfully");
                }
            }
        }
    }
    app.run().unwrap();

    Ok(())
}
