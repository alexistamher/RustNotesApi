use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{
    routes::{ books::books_config, notes::notes_config, users::users_config },
    util::token_manager::jwt_validator,
};

mod data;
mod routes;
mod domain;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::HttpServer
        ::new(|| {
            let jwt_authentication = HttpAuthentication::with_fn(jwt_validator);

            actix_web::App
                ::new()
                .wrap(Logger::default())
                .wrap(jwt_authentication)
                .wrap(Cors::default().allow_any_header().allow_any_method().allow_any_origin())
                .configure(notes_config)
                .configure(books_config)
                .configure(users_config)
        })
        .bind(("127.0.0.1", 3001))?
        .run().await
}
