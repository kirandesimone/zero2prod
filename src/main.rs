use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::config;
use zero2prod::startup;
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // this will set up the logger for everything info level or above
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    //Bubble up the error if we failed to bind the address
    // otherwise call await on our server
    let configuration = config::get_configuration().expect("Could not read configuration file");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("could not connect to DB");
    let addr = format!("127.0.0.1:{}", configuration.app_port);
    let listener = TcpListener::bind(addr)?;
    startup::run(listener, connection_pool)?.await
}
