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


// ----------
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
// ----------- (First endpoint)



// ----------- (Second thing)
#[derive(Deserialize)]
struct RequestForTokenCreation {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct ResponseForInstruction {
    program_id: String,
    accounts: Vec<ResponseForAccountMeta>,
    instruction_data: String,
}

#[derive(Serialize)]
struct ResponseForAccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

async fn create_token(Json(req): Json<RequestForTokenCreation>) -> Result<Json<SuccessResponse<ResponseForInstruction>>, (StatusCode, Json<ErrorResponse>)> {
    let mint_authority = Pubkey::from_str(&req.mint_authority).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid mint authority".to_string(),
        }))
    })?;
    
    let mint = Pubkey::from_str(&req.mint).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid mint address".to_string(),
        }))
    })?;

    let instruction = initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        req.decimals,
    ).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Failed to create instruction".to_string(),
        }))
    })?;

    let accounts: Vec<ResponseForAccountMeta> = instruction.accounts.iter().map(|acc| {
        ResponseForAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        }
    }).collect();

    let response = ResponseForInstruction {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}
// -----------(Second endpoint complete, working till here)



#[derive(Deserialize)]
struct MintTokenWaliRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(Json(req): Json<MintTokenWaliRequest>) -> Result<Json<SuccessResponse<ResponseForInstruction>>, (StatusCode, Json<ErrorResponse>)> {
    let mint = Pubkey::from_str(&req.mint).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Mai error hun :) -- (Mint in endpoint 3)".to_string(),
        }))
    })?;
    
    let destination = Pubkey::from_str(&req.destination).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from destination in endpoint 3".to_string(),
        }))
    })?;
    
    let authority = Pubkey::from_str(&req.authority).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from authority in endpoint 3".to_string(),
        }))
    })?;

    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        req.amount,
    ).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "instruction failure in endpoint3, check krrr bhai.. jsldiiii".to_string(),
        }))
    })?;

    let accounts: Vec<ResponseForAccountMeta> = instruction.accounts.iter().map(|acc| {
        ResponseForAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        }
    }).collect();

    let response = ResponseForInstruction {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token));
        // .route("/message/sign", post(sign_message))
        // .route("/message/verify", post(verify_message))
        // .route("/send/sol", post(send_sol))
        // .route("/send/token", post(send_token));

    let addr = SocketAddr::from(([127,0,0,1], 3000));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

