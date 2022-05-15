use crate::helpers::spawn_app;

// tokio::test is the the testing equivalent to #[test] attribute
#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    //bring in reqwest
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute call");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
