use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io::stdout;
use std::net::TcpListener;
use zero2prod::config;
use zero2prod::startup;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), stdout);
    telemetry::init_subscriber(subscriber);
    //Bubble up the error if we failed to bind the address
    // otherwise call await on our server
    let configuration = config::get_configuration().expect("Could not read configuration file");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Could not connect to DB");
    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(addr)?;
    startup::run(listener, connection_pool)?.await
}
