use actix_web::{test, App};
use mangabot_rs::config::Config;
use mangabot_rs::services::web::configure as web_configure;
use uuid::Uuid;
use std::sync::OnceLock;

static INIT: OnceLock<()> = OnceLock::new();

#[actix_web::test]
async fn test_valid_token_download() {
    let cfg = Config::load().unwrap();
    INIT.get_or_init(|| {
        mangabot_rs::utils::cache::init(&cfg).unwrap();
    });

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("sample.txt");
    tokio::fs::write(&file_path, b"hello").await.unwrap();

    let token = Uuid::new_v4().to_string();
    mangabot_rs::utils::cache::download_token_cache()
        .insert(token.clone(), file_path.to_string_lossy().to_string())
        .await;

    let app = test::init_service(App::new().configure(web_configure)).await;
    let req = test::TestRequest::get()
        .uri(&format!("/download?token={}", token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let cd = resp.headers().get(actix_web::http::header::CONTENT_DISPOSITION).unwrap();
    let cd_str = cd.to_str().unwrap();
    assert!(cd_str.contains("attachment"));
}

#[actix_web::test]
async fn test_invalid_token_400() {
    let cfg = Config::load().unwrap();
    INIT.get_or_init(|| {
        mangabot_rs::utils::cache::init(&cfg).unwrap();
    });

    let app = test::init_service(App::new().configure(web_configure)).await;
    let req = test::TestRequest::get()
        .uri("/download?token=not-a-uuid")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_missing_file_404() {
    let cfg = Config::load().unwrap();
    INIT.get_or_init(|| {
        mangabot_rs::utils::cache::init(&cfg).unwrap();
    });

    let token = Uuid::new_v4().to_string();
    mangabot_rs::utils::cache::download_token_cache()
        .insert(token.clone(), "/tmp/this/does/not/exist.zip".to_string())
        .await;

    let app = test::init_service(App::new().configure(web_configure)).await;
    let req = test::TestRequest::get()
        .uri(&format!("/download?token={}", token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
