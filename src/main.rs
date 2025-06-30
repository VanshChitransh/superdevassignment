use axum::{
    routing::post,
    Router, 
    Json, 
    http::StatusCode,
    extract::rejection::JsonRejection,
    response::Response,
};

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


async fn extract_json<T>(payload: Result<Json<T>, JsonRejection>) -> Result<T, (StatusCode, Json<ErrorResponse>)>
where
    T: serde::de::DeserializeOwned,
{
    match payload {
        Ok(Json(data)) => Ok(data),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))),
    }
}


fn is_valid_base58(s: &str) -> bool {
    !s.trim().is_empty() && bs58::decode(s).into_vec().is_ok()
}

fn is_valid_base64(s: &str) -> bool {
    !s.trim().is_empty() && general_purpose::STANDARD.decode(s).is_ok()
}


fn is_valid_pubkey(s: &str) -> bool {
    !s.trim().is_empty() && Pubkey::from_str(s).is_ok()
}

fn is_suspicious_text(s: &str) -> bool {
    let s = s.trim();
    
    if s.is_empty() {
        return true;
    }
    
    
    if s.len() > 1000 {
        return true;
    }
    
   
    if s.contains('\0') || s.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
        return true;
    }
    
    
    let suspicious_patterns = [
        "drop table", "delete from", "insert into", "update set", 
        "union select", "' or '", "\" or \"", "; --", "/*", "*/",
        "<script", "</script", "javascript:", "data:", "vbscript:",
        "onload=", "onerror=", "onclick=", "../", "..\\",
    ];
    
    let lower_s = s.to_lowercase();
    for pattern in &suspicious_patterns {
        if lower_s.contains(pattern) {
            return true;
        }
    }
    
    false
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
    mint_authority: Option<String>,
    mint: Option<String>,
    decimals: Option<u8>,
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

async fn create_token(payload: Result<Json<RequestForTokenCreation>, JsonRejection>) -> Result<Json<SuccessResponse<ResponseForInstruction>>, (StatusCode, Json<ErrorResponse>)> {
    let req = extract_json(payload).await?;
    
    let mint_authority_str = req.mint_authority.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let mint_str = req.mint.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let decimals = req.decimals.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;

    if is_suspicious_text(mint_authority_str) || is_suspicious_text(mint_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    if decimals > 9 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid decimals value".to_string(),
        })));
    }

    if !is_valid_pubkey(mint_authority_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid mint authority".to_string(),
        })));
    }
    
    if !is_valid_pubkey(mint_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid mint address".to_string(),
        })));
    }

    let mint_authority = Pubkey::from_str(mint_authority_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid mint authority".to_string(),
        }))
    })?;
    
    let mint = Pubkey::from_str(mint_str).map_err(|_| {
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
        decimals,
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
    mint: Option<String>,
    destination: Option<String>,
    authority: Option<String>,
    amount: Option<u64>,
}

async fn mint_token(payload: Result<Json<MintTokenWaliRequest>, JsonRejection>) -> Result<Json<SuccessResponse<ResponseForInstruction>>, (StatusCode, Json<ErrorResponse>)> {
    let req = extract_json(payload).await?;
    
    let mint_str = req.mint.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let destination_str = req.destination.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let authority_str = req.authority.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let amount = req.amount.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;

    if is_suspicious_text(mint_str) || is_suspicious_text(destination_str) || is_suspicious_text(authority_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    if !is_valid_pubkey(mint_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Mai error hun :) -- (Mint in endpoint 3)".to_string(),
        })));
    }
    
    if !is_valid_pubkey(destination_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from destination in endpoint 3".to_string(),
        })));
    }
    
    if !is_valid_pubkey(authority_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from authority in endpoint 3".to_string(),
        })));
    }

    let mint = Pubkey::from_str(mint_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Mai error hun :) -- (Mint in endpoint 3)".to_string(),
        }))
    })?;
    
    let destination = Pubkey::from_str(destination_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from destination in endpoint 3".to_string(),
        }))
    })?;
    
    let authority = Pubkey::from_str(authority_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Error from authority in endpoint 3".to_string(),
        }))
    })?;

    if amount == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount must be greater than 0".to_string(),
        })));
    }

    if amount > u64::MAX / 2 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount too large".to_string(),
        })));
    }

    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        amount,
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
    message: Option<String>,
    secret: Option<String>,
}

#[derive(Serialize)]
struct SignatureResponse {
    signature: String,
    public_key: String,
    message: String,
}


async fn sign_message(payload: Result<Json<SignMessageRequest>, JsonRejection>) -> Result<Json<SuccessResponse<SignatureResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let req = extract_json(payload).await?;
    
    let message = req.message.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let secret = req.secret.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;

    if is_suspicious_text(message) || is_suspicious_text(secret) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    if !is_valid_base58(secret) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "key format theek karo".to_string(),
        })));
    }

    let secret_bytes = bs58::decode(secret).into_vec().map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "key format theek karo".to_string(),
        }))
    })?;

    if secret_bytes.len() != 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid secret key".to_string(),
        })));
    }

    let keypair = Keypair::from_bytes(&secret_bytes).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid secret key".to_string(),
        }))
    })?;

    let message_bytes = message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    let response = SignatureResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: message.clone(),
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
    message: Option<String>,
    signature: Option<String>,
    pubkey: Option<String>,
}

#[derive(Serialize)]
struct VerificationResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

async fn verify_message(payload: Result<Json<VerifyMessageRequest>, JsonRejection>) -> Result<Json<SuccessResponse<VerificationResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let req = extract_json(payload).await?;
    
    let message = req.message.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let signature_str = req.signature.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let pubkey_str = req.pubkey.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;

    if is_suspicious_text(message) || is_suspicious_text(signature_str) || is_suspicious_text(pubkey_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }

    if !is_valid_pubkey(pubkey_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid public key".to_string(),
        })));
    }

    if !is_valid_base64(signature_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature format".to_string(),
        })));
    }

    let pubkey = Pubkey::from_str(pubkey_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid public key".to_string(),
        }))
    })?;

    let signature_bytes = general_purpose::STANDARD.decode(signature_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature format".to_string(),
        }))
    })?;

    if signature_bytes.len() != 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature".to_string(),
        })));
    }

    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid signature".to_string(),
        }))
    })?;

    let message_bytes = message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    let response = VerificationResponse {
        valid: is_valid,
        message: message.clone(),
        pubkey: pubkey_str.clone(),
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
    from: Option<String>,
    to: Option<String>,
    lamports: Option<u64>,
}

#[derive(Serialize)]
struct SolTransferResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

async fn send_sol(payload: Result<Json<SendSolRequest>, JsonRejection>) -> Result<Json<SuccessResponse<SolTransferResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let req = extract_json(payload).await?;
    
    let from_str = req.from.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let to_str = req.to.as_ref().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;
    
    let lamports = req.lamports.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        }))
    })?;

    if is_suspicious_text(from_str) || is_suspicious_text(to_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Missing required fields".to_string(),
        })));
    }


    if !is_valid_pubkey(from_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid from address".to_string(),
        })));
    }

    if !is_valid_pubkey(to_str) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid to address".to_string(),
        })));
    }

    let from_pubkey = Pubkey::from_str(from_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid from address".to_string(),
        }))
    })?;

    let to_pubkey = Pubkey::from_str(to_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Invalid to address".to_string(),
        }))
    })?;

    if lamports == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount must be greater than 0".to_string(),
        })));
    }

    if lamports > 1_000_000_000_000_000_000 { // 1 billion SOL in lamports
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Amount too large".to_string(),
        })));
    }

    if from_pubkey == to_pubkey {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            success: false,
            error: "Cannot send to same address".to_string(),
        })));
    }

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

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
    destination: Option<String>,
    mint: Option<String>,
    owner: Option<String>,
    amount: Option<u64>,
}

async fn send_token(payload: Result<Json<SendTokenRequest>, JsonRejection>) -> Result<Json<SuccessResponse<()>>, (StatusCode, Json<ErrorResponse>)> {
    let _req = extract_json(payload).await?;
    
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