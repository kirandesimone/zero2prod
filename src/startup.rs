use crate::config::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::*;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_database_configuration(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("could not parse sender email");
        let base_url = configuration.email_client.base_url.clone();
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.sg_key.clone(),
            sender_email,
            base_url,
            timeout,
        );
        let addr = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&addr)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_database_configuration(db_config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db_config.connection_with_db())
}

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    //wrap in web::Data which boils down to an Arc smart pointer
    let connection = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone())
            .app_data(email_client.clone()) // register the db connection as part of app state
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
