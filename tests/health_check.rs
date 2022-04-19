////////////////////////////////
// INTEGRATION TEST FOR APIS //
//////////////////////////////

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::{sink, stdout};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::config::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::*;
use zero2prod::{startup, telemetry};
use secrecy::ExposeSecret;

// ensures that the 'tracing' stack is initialised once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let env_filter = "test".to_string();
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

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    //binding to port 0 will tell the OS to bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bin to port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("could not load config");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

// for creating brand-new logical database for each integration test
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create Database
    let mut connection = PgConnection::connect(&config.connection_string_without_db_name().expose_secret())
        .await
        .expect("Failed to connect to DB with no name");
    connection
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // migrate the database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Couldn't connect to the db");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool
}

// tokio::test is the the testing equivalent to #[test] attribute
#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    //bring in reqwest
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute call");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscriber_returns_a_400_for_invalid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "No email provided"),
        ("email=ursula_le_guin%40gmail.com", "No name provided"),
        ("", "No data provided"),
    ];

    for (invalid_data, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_data)
            .send()
            .await
            .expect("Could not complete requests");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error
        );
    }
}
