#[macro_use]
extern crate rocket;

mod rauc;

use rauc::{RaucBundleInfo, RaucClient, RaucMode, RaucStatus};
use rocket::data::{Limits, ToByteUnit};
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::{ContentType, Status};
use rocket::response::content::RawHtml;
use rocket::response::stream::TextStream;
use rocket::serde::json::Json;
use rocket::State;
use rust_embed::RustEmbed;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::fs;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

#[derive(FromForm)]
struct Upload<'r> {
    file: TempFile<'r>,
}

struct AppConfig {
    upload_dir: PathBuf,
    bundle_path: PathBuf,
    web_ui_title: String,
    web_ui_primary_color: String,
    web_ui_background_color: String,
    web_ui_foreground_color: String,
}

#[get("/")]
fn index(config: &State<AppConfig>) -> RawHtml<String> {
    let html = Asset::get("index.html")
        .map(|content| String::from_utf8_lossy(&content.data).to_string())
        .unwrap_or_else(|| "<html><body>Index not found</body></html>".to_string());

    // Inject title and CSS variables at runtime
    let head_injection = format!(
        r#"<title>{}</title>
<style>
:root {{
    --primary-color: {};
    --background-color: {};
    --foreground-color: {};
    --project-name: '{}';
}}
</style>"#,
        config.web_ui_title,
        config.web_ui_primary_color,
        config.web_ui_background_color,
        config.web_ui_foreground_color,
        config.web_ui_title
    );

    let injected_html = html.replace("</head>", &format!("{}</head>", head_injection));
    RawHtml(injected_html)
}

#[get("/<file..>", rank = 10)]
fn static_files(file: PathBuf) -> Option<(ContentType, Vec<u8>)> {
    let filename = file.to_str()?;
    let asset = Asset::get(filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Binary);

    Some((content_type, asset.data.to_vec()))
}

#[get("/api/status")]
async fn api_status(client: &State<RaucClient>) -> Result<Json<RaucStatus>, (Status, String)> {
    let status = client
        .get_status()
        .await
        .map_err(|e| (Status::InternalServerError, e))?;
    Ok(Json(status))
}

#[post("/api/upload", data = "<upload>")]
async fn api_upload(
    mut upload: Form<Upload<'_>>,
    config: &State<AppConfig>,
    client: &State<RaucClient>,
) -> Result<String, (Status, String)> {
    // Ensure the upload directory exists
    fs::create_dir_all(&config.upload_dir).await.map_err(|e| {
        (
            Status::InternalServerError,
            format!("Failed to create upload directory: {}", e),
        )
    })?;

    // Save the uploaded file
    upload
        .file
        .persist_to(&config.bundle_path)
        .await
        .map_err(|e| {
            (
                Status::InternalServerError,
                format!("Failed to save uploaded file: {}", e),
            )
        })?;

    // Verify file was written correctly
    let metadata = fs::metadata(&config.bundle_path).await.map_err(|e| {
        (
            Status::InternalServerError,
            format!("Failed to verify uploaded file: {}", e),
        )
    })?;

    let file_size = metadata.len();

    let bundle_path_str = config.bundle_path.to_str().ok_or_else(|| {
        (
            Status::InternalServerError,
            "Invalid bundle path".to_string(),
        )
    })?;

    // Copy file to target in development mode
    let copy_result = client
        .copy_file_to_target(bundle_path_str, bundle_path_str)
        .await
        .map_err(|e| (Status::InternalServerError, e))?;

    Ok(format!(
        "Bundle uploaded successfully to {} ({} bytes). {}",
        config.bundle_path.display(),
        file_size,
        copy_result
    ))
}

#[get("/api/bundle-info")]
async fn api_bundle_info(
    client: &State<RaucClient>,
    config: &State<AppConfig>,
) -> Result<Json<RaucBundleInfo>, (Status, String)> {
    let bundle_path_str = config.bundle_path.to_str().ok_or_else(|| {
        (
            Status::InternalServerError,
            "Invalid bundle path".to_string(),
        )
    })?;

    let info = client
        .get_bundle_info(bundle_path_str)
        .await
        .map_err(|e| (Status::InternalServerError, e))?;
    Ok(Json(info))
}

#[get("/api/install")]
fn api_install(client: &State<RaucClient>, config: &State<AppConfig>) -> TextStream![String] {
    let bundle_path = config
        .bundle_path
        .to_str()
        .unwrap_or("/tmp/rauc-bundles/upload_bundle.raucb")
        .to_string();
    let client = client.inner().clone();

    TextStream! {
        match client.install_bundle(&bundle_path).await {
            Ok(stream) => {
                for await result in stream {
                    match result {
                        Ok(line) => yield line,
                        Err(e) => yield format!("[ERROR] {}\n", e),
                    }
                }
            }
            Err(e) => {
                yield format!("[ERROR] Failed to start installation: {}\n", e);
            }
        }
    }
}

#[post("/api/reboot")]
async fn api_reboot(client: &State<RaucClient>) -> Result<String, (Status, String)> {
    client
        .reboot()
        .await
        .map_err(|e| (Status::InternalServerError, e))
}

#[launch]
fn rocket() -> _ {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Determine mode based on environment variables
    let mode = match (env::var("SSH_HOST"), env::var("SSH_PASSWORD")) {
        (Ok(ssh_host), Ok(ssh_password)) => {
            println!("Running in DEVELOPMENT mode with SSH");
            RaucMode::Development {
                ssh_host,
                ssh_password,
            }
        }
        _ => {
            if cfg!(debug_assertions) {
                println!("Running in DEBUG mode with direct rauc binary (no SSH credentials)");
            } else {
                println!("Running in PRODUCTION mode with direct rauc binary");
            }
            RaucMode::Production
        }
    };

    let rauc_client = RaucClient::new(mode.clone());

    // Get upload directory from env or use default
    let upload_dir = env::var("UPLOAD_TMP_DIR")
        .unwrap_or_else(|_| "/tmp/rauc-bundles".to_string())
        .into();

    let bundle_path = env::var("UPLOAD_TMP_DIR")
        .map(|dir| PathBuf::from(dir).join("upload_bundle.raucb"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/rauc-bundles/upload_bundle.raucb"));

    // Get theming configuration
    let web_ui_title = env::var("WEB_UI_TITLE").unwrap_or_else(|_| "Firmware Updater".to_string());
    let web_ui_primary_color =
        env::var("WEB_UI_PRIMARY_COLOR").unwrap_or_else(|_| "rgb(59, 130, 246)".to_string()); // blue-500
    let web_ui_background_color =
        env::var("WEB_UI_BACKGROUND_COLOR").unwrap_or_else(|_| "rgb(249, 250, 251)".to_string()); // gray-50
    let web_ui_foreground_color =
        env::var("WEB_UI_FOREGROUND_COLOR").unwrap_or_else(|_| "rgb(17, 24, 39)".to_string()); // gray-900

    let app_config = AppConfig {
        upload_dir,
        bundle_path,
        web_ui_title,
        web_ui_primary_color,
        web_ui_background_color,
        web_ui_foreground_color,
    };

    // Get port from environment or use default
    let port = env::var("PORT")
        .or_else(|_| env::var("ROCKET_PORT"))
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);

    let address = (match mode {
        RaucMode::Production { .. } => "0.0.0.0",
        _ => "127.0.0.1",
    })
    .parse()
    .unwrap();
    rocket::build()
        .manage(rauc_client)
        .manage(app_config)
        .configure(rocket::Config {
            limits: Limits::default()
                .limit("file", 512.mebibytes())
                .limit("data-form", 512.mebibytes()),
            address,
            port,
            ..Default::default()
        })
        .mount(
            "/",
            routes![
                index,
                static_files,
                api_status,
                api_upload,
                api_bundle_info,
                api_install,
                api_reboot
            ],
        )
}
