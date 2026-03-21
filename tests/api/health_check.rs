use crate::helpers::init;

#[derive(serde::Deserialize)]
struct ResponseBody {
    msg: String
}

#[tokio::test]
async fn health_check_returns_200_and_ok_msg() {
    let (app, _, http_client, _, _) = init().await;

    // Act
    let response = http_client
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