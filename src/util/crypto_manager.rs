use aes_gcm::{ aead::{ rand_core::RngCore, Aead, OsRng }, Aes256Gcm, Key, KeyInit, Nonce };
use p256::{
    ecdh::diffie_hellman,
    elliptic_curve::sec1::ToEncodedPoint,
    pkcs8::DecodePublicKey,
    PublicKey,
    SecretKey,
};
use serde::{ Deserialize, Serialize };
use base64::{ engine::general_purpose::STANDARD as base64, Engine as _ };
use sha2::{Sha256, Digest};
use uuid::Uuid;
use crate::util::session_manager::CACHE;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyExchangeRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyExchangeResponse {
    pub public_key: String,
    pub session_id: String,
    pub shared_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptResponse {
    pub encrypted_data: String,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptRequest {
    pub session_id: String,
    pub encrypted_data: String,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptResponse {
    pub decrypted_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub client_public_key: String,
    pub server_public_key: String,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub client_public_key: PublicKey,
    pub server_secret_key: SecretKey,
    pub server_public_key: PublicKey,
    pub shared_secret: Vec<u8>,
    pub aes_key: [u8; 32],
}

pub fn generate_key_pair() -> (SecretKey, PublicKey) {
    let secret_key = SecretKey::random(&mut OsRng);
    let public_key = secret_key.public_key();
    (secret_key, public_key)
}

pub fn public_key_from_base64(public_key_b64: &str) -> Result<PublicKey, String> {
    let public_key_bytes = base64
        .decode(public_key_b64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Try X.509 DER format first (Android sends this format)
    if let Ok(public_key) = PublicKey::from_public_key_der(&public_key_bytes) {
        return Ok(public_key);
    }

    // Fallback to SEC1 format
    PublicKey::from_sec1_bytes(&public_key_bytes).map_err(|e|
        format!("Failed to parse public key in both X.509 and SEC1 formats: {}", e)
    )
}

pub fn public_key_to_base64(public_key: &PublicKey) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    base64.encode(encoded_point.as_bytes())
}

pub fn derive_shared_secret(
    server_secret: &SecretKey,
    client_public: &PublicKey
) -> Result<Vec<u8>, String> {
    let shared_secret = diffie_hellman(
        server_secret.to_nonzero_scalar(),
        client_public.as_affine()
    );
    Ok(shared_secret.raw_secret_bytes().to_vec())
}

pub fn derive_aes_key(shared_secret: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(shared_secret);
    hasher.update(b"e2e_encryption_key");
    let result = hasher.finalize();
    result.into()
}

pub fn encrypt_aes_gcm(data: &str, key: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt_aes_gcm(
    encrypted_data: &[u8],
    nonce: &[u8],
    key: &[u8; 32]
) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Failed to convert to UTF-8: {}", e))
}

fn generate_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; 12]; // 96-bit nonce for AES-GCM
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn base64_encode(data: &[u8]) -> String {
    base64.encode(data)
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    base64.decode(data).map_err(|e| format!("Base64 decode error: {}", e))
}

pub fn key_exchange(client_public_key: String) -> Result<KeyExchangeResponse, String> {
    let Ok(client_public_key) = public_key_from_base64(&client_public_key) else {
        return Err("Invalid public key".to_string());
    };

    // Generate server key pair
    let (server_secret_key, server_public_key) = generate_key_pair();

    // Derive shared secret
    let shared_secret = match derive_shared_secret(&server_secret_key, &client_public_key) {
        Ok(secret) => secret,
        Err(e) => {
            return Err("Failed to derive shared secret: {}".to_string());
        }
    };

    // Derive AES key
    let aes_key = derive_aes_key(&shared_secret);

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();

    // Store session info
    let session_info = SessionInfo {
        session_id: session_id.clone(),
        client_public_key,
        server_secret_key,
        server_public_key: server_public_key.clone(),
        shared_secret: shared_secret.clone(),
        aes_key,
    };

    {
        let sessions_locked = CACHE.sessions.lock();
        let mut sessions = sessions_locked.unwrap();
        sessions.insert(session_id.clone(), session_info);
    }

    Ok(KeyExchangeResponse {
        public_key: public_key_to_base64(&server_public_key),
        session_id,
        shared_secret: base64_encode(&shared_secret),
    })
}
