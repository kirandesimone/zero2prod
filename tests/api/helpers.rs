use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::{sink, stdout};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::config::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::{get_database_configuration, Application};
use zero2prod::telemetry::init_subscriber;
use zero2prod::{startup, telemetry};

// ensures that the 'tracing' stack is initialised once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let env_filter = "test".to_string();
    let subscriber_name = "ZERO2PROD_TEST".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(subscriber_name, env_filter, stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(subscriber_name, env_filter, sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute query")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // randomize configurations for test isolation
    let mut configuration = get_configuration().expect("could not load config");
    configuration.database.database_name = Uuid::new_v4().to_string();
    configuration.application.port = 0;
    let db_config = configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://127.0.0.1:{}", application.get_port());

    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: get_database_configuration(&configuration.database),
    }
}

// for creating brand-new logical database for each integration test
async fn configure_database(db_config: &DatabaseSettings) -> PgPool {
    // create Database
    let mut connection = PgConnection::connect_with(&db_config.connection_without_db())
        .await
        .expect("Failed to connect to DB with no name");
    connection
        .execute(format!(r#"create database "{}";"#, db_config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // migrate the database
    let connection_pool = PgPool::connect_with(db_config.connection_with_db())
        .await
        .expect("Couldn't connect to the db");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool
}
