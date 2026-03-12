use crate::helpers::spawn_app;

#[derive(serde::Deserialize)]
struct ResponseBody {
    msg: String
}

#[tokio::test]
async fn health_check_returns_200_and_ok_msg() {
    let app = spawn_app(None).await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/api/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    let status = response.status();
    let body_json: ResponseBody = response
        .json()
        .await
        .unwrap();

    // Assert
    assert!(status.is_success());
    assert!(body_json.msg == String::from("OK"))
}