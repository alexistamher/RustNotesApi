use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::{EitherBody, MessageBody},
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    web,
};
use bytes::BytesMut;
use futures_util::StreamExt;
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};

use crate::util::session_manager::check_session;
use rust_cipher_lib::crypto_manager::{
    base64_decode, base64_decode_bytes, base64_encode, decrypt_aes_gcm, encrypt_aes_gcm,
};

pub struct CryptoMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for CryptoMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CryptoMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CryptoMiddleware {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct CryptoMiddleware<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CryptoMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            if req.method() != actix_web::http::Method::POST
                || !req.headers().contains_key("x-session-key")
            {
                return srv.call(req).await.map(|res| res.map_into_left_body());
            }

            let headers = req.headers().clone();

            let session_id = match headers.get("x-session-key").and_then(|v| v.to_str().ok()) {
                Some(id) => id,
                None => {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Invalid or missing x-session-key header",
                    ));
                }
            };

            let nonce_encoded = match headers.get("x-nonce").and_then(|v| v.to_str().ok()) {
                Some(id) => id,
                None => {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Invalid or missing x-nonce header",
                    ));
                }
            };

            let session = match check_session(session_id) {
                Some(session) => session,
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Session not found or expired",
                    ));
                }
            };
            let aes_key = session.aes_key;

            let mut body = BytesMut::new();
            let mut payload = req.take_payload();
            while let Some(chunk) = payload.next().await {
                body.extend_from_slice(&chunk?);
            }

            let nonce = base64_decode(nonce_encoded)
                .map_err(|_| actix_web::error::ErrorBadRequest("invalid Base64 for nonce"))?;
            let ciphertext = base64_decode_bytes(body.as_ref())
                .map_err(|_| actix_web::error::ErrorBadRequest("invalid Base64 for ciphertext"))?;

            let decrypted_body = decrypt_aes_gcm(&ciphertext, &nonce, &aes_key)
                .map_err(|_| actix_web::error::ErrorInternalServerError("decryption failed"))?;

            let (request, _original_payload) = req.into_parts();
            let new_payload = Payload::from(web::Bytes::from(decrypted_body));
            let new_req = ServiceRequest::from_parts(request, new_payload);

            let res = srv.call(new_req).await;
            if res.is_err() {
                return res.map(|res| res.map_into_left_body());
            }
            let res = res.unwrap();
            let (req_part, http_res) = res.into_parts();
            let (res_head, res_body) = http_res.into_parts();

            if !res_head.status().is_success() {
                return Ok(ServiceResponse::new(req_part, res_head.set_body(res_body))
                    .map_into_left_body());
            }
            let body_bytes = actix_web::body::to_bytes(res_body)
                .await
                .map_err(Into::into)?;
            if body_bytes.is_empty() {
                let empty_res = HttpResponse::build(res_head.status()).body("");
                return Ok(ServiceResponse::new(req_part, empty_res).map_into_right_body());
            }
            let body_str = std::str::from_utf8(&body_bytes).map_err(|_| {
                actix_web::error::ErrorInternalServerError("response body is not valid UTF-8")
            })?;

            let (ciphertext, nonce) = encrypt_aes_gcm(body_str, &aes_key).map_err(|_| {
                actix_web::error::ErrorInternalServerError("response encryption failed")
            })?;

            let encoded_body_response = base64_encode(&ciphertext);

            let mut new_res = HttpResponse::build(res_head.status());
            for (name, value) in res_head.headers() {
                new_res.insert_header((name.clone(), value.clone()));
            }
            new_res.insert_header(("x-nonce", base64_encode(&nonce)));

            let final_res = new_res.body(encoded_body_response);
            Ok(ServiceResponse::new(req_part, final_res).map_into_right_body())
        })
    }
}
