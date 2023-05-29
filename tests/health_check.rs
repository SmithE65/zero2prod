use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let configuration = get_configuration().expect("Failed to read config.");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to database.");
    let address = spawn_app();
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
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app();
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

#[allow(clippy::let_underscore_future)]
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address.");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{port}")
}
