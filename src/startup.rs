use crate::routes::*;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    //wrap in web::Data which boils down to an Arc smart pointer
    let connection = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone()) // register the db connection as part of app state
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
