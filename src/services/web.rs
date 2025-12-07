use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use crate::config::Config;
use crate::utils::cache;
use mime_guess::MimeGuess;
use serde::Deserialize;
use tokio::fs;
use tracing::{error, info};

#[derive(Deserialize)]
struct DownloadQuery {
    token: String,
}

async fn handle_download(req: HttpRequest, query: web::Query<DownloadQuery>) -> actix_web::Result<HttpResponse> {
    let token_str = query.token.trim();
    if uuid::Uuid::parse_str(token_str).is_err() {
        error!(token = token_str, "invalid token format");
        return Ok(HttpResponse::BadRequest().body("invalid token"));
    }

    let cache = cache::download_token_cache();
    let path_opt = cache.get(token_str).await;
    if path_opt.is_none() {
        error!(token = token_str, "token not found in cache");
        return Ok(HttpResponse::NotFound().finish());
    }
    let path = path_opt.unwrap();

    if fs::metadata(&path).await.is_err() {
        error!(path = %path, "file not found");
        return Ok(HttpResponse::NotFound().finish());
    }

    let file = match NamedFile::open_async(&path).await {
        Ok(f) => f,
        Err(e) => {
            error!(path = %path, error = %e, "file open error");
            return Ok(HttpResponse::InternalServerError().body("file open error"));
        }
    };

    let filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download");

    let cd = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(String::from(filename))],
    };

    let mime = MimeGuess::from_path(&path).first_or_octet_stream();
    let file = file.set_content_type(mime).set_content_disposition(cd);
    Ok(file.into_response(&req))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/download", web::get().to(handle_download));
}

pub fn start(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ("0.0.0.0", config.server.port);
    info!(port = config.server.port, "starting web server");
    std::thread::spawn(move || {
        let sys = actix_web::rt::System::new();
        let _ = sys.block_on(async move {
            let _ = HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(web::Data::new(config.clone()))
                    .configure(configure)
            })
            .bind(addr)
            .expect("bind failed")
            .run()
            .await;
        });
    });
    Ok(())
}
