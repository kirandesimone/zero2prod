// tokio::test is the the testing equivalent to #[test] attribute

#[tokio::test]
async fn health_check_works() {
    spawn_app();
    //bring in reqwest
    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to execute call");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");

    let _ = tokio::spawn(server);
}
