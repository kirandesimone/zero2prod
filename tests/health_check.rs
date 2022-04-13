////////////////////////////////
// INTEGRATION TEST FOR APIS //
//////////////////////////////

use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::config;
use zero2prod::startup;

fn spawn_app() -> String {
    //binding to port 0 will tell the OS to bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bin to port");
    let port = listener.local_addr().unwrap().port();
    let server = startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

// tokio::test is the the testing equivalent to #[test] attribute
#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    //bring in reqwest
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute call");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    let address = spawn_app();
    let configuration = config::get_configuration().expect("failed to load config file");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscriber_returns_a_400_for_invalid_form_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "No email provided"),
        ("email=ursula_le_guin%40gmail.com", "No name provided"),
        ("", "No data provided"),
    ];

    for (invalid_data, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
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
