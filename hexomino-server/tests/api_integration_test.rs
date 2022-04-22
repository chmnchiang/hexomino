use std::net::TcpListener;

use anyhow::Result;
use api::{AuthPayload, AuthResponse};
use hexomino_server::make_app;

fn spawn_server() -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?.to_string();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(make_app())
            .await
            .unwrap();
    });
    Ok(addr)
}

#[tokio::test]
async fn when_provide_correct_cred_login_succeeds() -> Result<()> {
    let addr = spawn_server()?;
    let payload = AuthPayload {
        username: "hao123".to_string(),
        password: "hao123".to_string(),
    };
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{addr}/api/login"))
        .json(&payload)
        .send()
        .await?;
    assert_eq!(res.status(), 200);
    let resp = res.json::<AuthResponse>().await?;

    let res = client
        .get(format!("http://{addr}/api/protect"))
        .header("Authorization", format!("Bearer {}", resp.token))
        .send()
        .await?;
    assert_eq!(res.status(), 200);

    Ok(())
}

#[tokio::test]
async fn when_provide_incorrect_cred_login_fails() -> Result<()> {
    let addr = spawn_server()?;

    let payload = AuthPayload {
        username: "hao123".to_string(),
        password: "hao124".to_string(),
    };
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{addr}/api/login"))
        .json(&payload)
        .send()
        .await?;
    assert_eq!(res.status(), 401);

    Ok(())
}

#[tokio::test]
async fn when_provide_incorrect_cred_blocks_access_to_protect() -> Result<()> {
    let addr = spawn_server()?;
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{addr}/api/protect"))
        .header("Authorization", "Bearer Hao123")
        .send()
        .await?;
    assert_eq!(res.status(), 401);

    Ok(())
}
