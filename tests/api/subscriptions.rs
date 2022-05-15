use crate::helpers::spawn_app;

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscription(body.into()).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}


#[tokio::test]
async fn subscriber_returns_a_400_for_invalid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "No email provided"),
        ("email=ursula_le_guin%40gmail.com", "No name provided"),
        ("", "No data provided"),
    ];

    for (invalid_data, error) in test_cases {
        let response = app.post_subscription(invalid_data.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Rachel&email=", "empty email"),
        ("name=&email=bob%40gmail.com", "empty name"),
        ("name=&email=", "empty name and email"),
        ("name=Tim&email=this-is-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscription(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BAD REQUEST when the payload was {}",
            description
        );
    }
}
