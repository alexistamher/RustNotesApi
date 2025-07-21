use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{
    routes::{
        books::books_config,
        crypto_middleware::CryptoMiddlewareFactory,
        notes::notes_config,
        session::session_config,
        users::users_config,
    },
    util::{
        config::ServerConfig,
        token_manager::jwt_validator,
    },
};

mod data;
mod routes;
mod domain;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let config = envy
        ::prefixed("SERVER_")
        .from_env::<ServerConfig>()
        .expect("server config missing");
    actix_web::HttpServer
        ::new(|| {
            let jwt_authentication = HttpAuthentication::with_fn(jwt_validator);

            actix_web::App
                ::new()
                .wrap(Logger::default())
                .wrap(CryptoMiddlewareFactory)
                .wrap(jwt_authentication)
                .wrap(Cors::default().allow_any_header().allow_any_method().allow_any_origin())
                .configure(notes_config)
                .configure(books_config)
                .configure(users_config)
                .configure(session_config)
        })
        .bind((config.host, config.port))?
        .run().await
}
