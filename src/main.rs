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
use base64::{Engine as _, engine::general_purpose};



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
    // Check for empty strings
    if req.mint_authority.trim().is_empty() || req.mint.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

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
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}
// -----------(Second endpoint complete, working till here)



// -------------------- THird one is here

#[derive(Deserialize)]
struct MintTokenWaliRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(Json(req): Json<MintTokenWaliRequest>) -> Result<Json<SuccessResponse<ResponseForInstruction>>, (StatusCode, Json<ErrorResponse>)> {
    // Check for empty strings
    if req.mint.trim().is_empty() || req.destination.trim().is_empty() || req.authority.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

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

    // Add validation for zero amount
    if req.amount == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount must be greater than 0".to_string(),
        })));
    }

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
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}


// ------------------ THird one completess here(Endpoint 3 is dpne);


//// Fourth one start here!!!

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignatureResponse {
    signature: String,
    public_key: String,
    message: String,
}


async fn sign_message(Json(req): Json<SignMessageRequest>) -> Result<Json<SuccessResponse<SignatureResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Check for empty/missing fields
    if req.message.trim().is_empty() || req.secret.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    let secret_bytes = bs58::decode(&req.secret).into_vec().map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "key format theek karo".to_string(),
        }))
    })?;

    let keypair = Keypair::from_bytes(&secret_bytes).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid secret key".to_string(),
        }))
    })?;

    let message_bytes = req.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    let response = SignatureResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: req.message,
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}


// Only 7 are working!!!!
// fourht end here (Endpoint 4 end's here)


// -----------------------




// enpoint 5
#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
struct VerificationResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

async fn verify_message(Json(req): Json<VerifyMessageRequest>) -> Result<Json<SuccessResponse<VerificationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Check for empty/missing fields
    if req.message.trim().is_empty() || req.signature.trim().is_empty() || req.pubkey.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    let pubkey = Pubkey::from_str(&req.pubkey).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid public key".to_string(),
        }))
    })?;

    let signature_bytes = general_purpose::STANDARD.decode(&req.signature).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature format".to_string(),
        }))
    })?;

    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature".to_string(),
        }))
    })?;

    let message_bytes = req.message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    let response = VerificationResponse {
        valid: is_valid,
        message: req.message,
        pubkey: req.pubkey,
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}



// endpoint 5
// ------------------------------------





// ---------------
// endpoitn 6


#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Serialize)]
struct SolTransferResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

async fn send_sol(Json(req): Json<SendSolRequest>) -> Result<Json<SuccessResponse<SolTransferResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Check for empty strings
    if req.from.trim().is_empty() || req.to.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    let from_pubkey = Pubkey::from_str(&req.from).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid from address".to_string(),
        }))
    })?;

    let to_pubkey = Pubkey::from_str(&req.to).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid to address".to_string(),
        }))
    })?;

    if req.lamports == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount must be greater than 0".to_string(),
        })));
    }

    // Check if from and to are the same
    if from_pubkey == to_pubkey {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Cannot send to same address".to_string(),
        })));
    }

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, req.lamports);

    let response = SolTransferResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction.accounts.iter().map(|acc| acc.pubkey.to_string()).collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(SuccessResponse {
        success: true,
        data: response,
    }))
}


// ---------------
// endpoint 7 - Send Token (Error response only as requested)

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

async fn send_token(Json(_req): Json<SendTokenRequest>) -> Result<Json<SuccessResponse<()>>, (StatusCode, Json<ErrorResponse>)> {
    Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
        success: false,
        error: "Token transfer endpoint not implemented".to_string(),
    })))
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