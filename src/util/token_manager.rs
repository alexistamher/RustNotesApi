use actix_web::{ dev::ServiceRequest, error, HttpRequest, HttpResponse };
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{ decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation };
use serde::{ Serialize, Deserialize };

const SECRET_KEY: &str = "notes_api_sample";

pub fn generate_token(user_id: &str) -> Token {
    let lifetime = 10i64;
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(SECRET_KEY.as_ref());
    let iat = chrono::Utc::now().timestamp();
    let exp = (chrono::Utc::now() + chrono::Duration::seconds(lifetime)).timestamp() as usize;
    let jti = uuid::Uuid::new_v4().to_string();
    let claims = Claims {
        iat,
        exp,
        jti,
        user_id: user_id.to_owned(),
    };
    let token = encode(&header, &claims, &key).expect("error on token generation");
    Token { token, lifetime }
}

pub fn check_token(token: &str) -> Option<Claims> {
    let validation = Validation::new(Algorithm::HS512);
    let decoding_key = DecodingKey::from_secret(SECRET_KEY.as_ref());
    let claims = decode::<Claims>(token, &decoding_key, &validation)
        .ok()
        .map(|t| t.claims)?;
    if (chrono::Utc::now().timestamp() as usize) >= claims.exp {
        return None;
    }
    Some(claims)

    // todo(agregar validación de lista negra)
}

pub fn revoke_creds(jti: &str) {
    // todo(agregar revocación de tokens con lista negra)
}

pub async fn jwt_validator(
    req: ServiceRequest,
    creds: Option<BearerAuth>
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let path = req.path();
    let paths = ["/user/login", "/user/register"];
    if paths.iter().any(|&p| path.eq(p)) {
        println!("started...");
        return Ok(req);
    }
    let Some(creds) = creds else {
        return Err((error::ErrorForbidden("bearer missing"), req));
    };
    let Some(claims) = check_token(creds.token()) else {
        return Err((error::ErrorUnauthorized("unauthorized"), req));
    };
    // todo!(crear validador para lista de tokens revocados)
    //exist_in_blacklist(claims.jti);

    Ok(req)
}

pub trait TokenExtractor<T> {
    fn token(self) -> Result<String, HttpResponse>;
}

impl TokenExtractor<HttpRequest> for HttpRequest {
    fn token(self) -> Result<String, HttpResponse> {
        let token_header = self
            .headers()
            .get("Authorization")
            .ok_or(HttpResponse::Unauthorized().finish())?;
        let string_token = format!("{:?}", token_header).replace('"', "");
        let token_slices = string_token.split(' ').collect::<Vec<&str>>();
        token_slices
            .last()
            .map(|&t| t.to_owned())
            .ok_or(HttpResponse::Unauthorized().finish())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub iat: i64,
    pub exp: usize,
    pub jti: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Token {
    token: String,
    lifetime: i64,
}
