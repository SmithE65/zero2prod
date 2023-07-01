use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let address = app.address;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{address}/subscribe"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let address = app.address;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=l2%20guin", "email missing from request"),
        (
            "email=ursula_le_guin%40gmail.com",
            "name missing from request",
        ),
        ("", "no data in request"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(format!("{address}/subscribe"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to make request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "Expected bad request when {error_message}"
        );
    }
}
