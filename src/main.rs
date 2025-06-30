use axum::{
    routing::post,
    Router, 
    Json, 
    http::StatusCode};

use serde::{
    Deserialize, 
    Serialize};

use solana_sdk::{
    signature::{Keypair, Signer, Signature},
    pubkey::Pubkey,
    system_instruction,
    instruction::{Instruction, AccountMeta},
};

use spl_token::instruction::{initialize_mint, 
    mint_to, 
    transfer};

use std::str::FromStr;
use std::net::SocketAddr;



#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

#[derive(Serialize)]
struct ResponseOfKeypair {
    pubkey: String,
    secret: String,
}

async fn generate_keypair() -> Json<SuccessResponse<ResponseOfKeypair>> {
    let keypair = Keypair::new();
    let response = ResponseOfKeypair {
        pubkey: keypair.pubkey().to_string(),
        secret: bs58::encode(keypair.to_bytes()).into_string(),
    };
    Json(SuccessResponse {
        success: true,
        data: response,
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token));

    let addr = SocketAddr::from(([127,0,0,1], 3000));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn main() {
    println!("Hello, world!");
}
