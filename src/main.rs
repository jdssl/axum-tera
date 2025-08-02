use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tera::{Context, Tera};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/**/*").map_err(|e| {
        eprintln!("Failed to load templates: {e}");
        e
    })?;
    let tera = Arc::new(Mutex::new(tera));

    let app = Router::new()
        .route("/static/{file}", get(static_handler))
        .route("/{name}", get(name))
        .fallback(handler_404)
        .with_state(tera.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        eprintln!("Failed to bind to address: {e}");
        e
    })?;

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| {
            eprintln!("Server error: {e}");
            e
        })?;

    Ok(())
}

async fn name(Path(name): Path<String>, State(tera): State<Arc<Mutex<Tera>>>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("name", &name);

    let tera = tera.lock().await;
    match tera.render("name.html", &context) {
        Ok(rendered) => Html(rendered).into_response(),
        Err(err) => {
            eprintln!("Template render error: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html::<String>("Failed to render page".into()),
            )
                .into_response()
        }
    }
}

async fn handler_404(State(tera): State<Arc<Mutex<Tera>>>) -> impl IntoResponse {
    let tera = tera.lock().await;
    let rendered = tera
        .render("404.html", &Context::new())
        .unwrap_or_else(|err| {
            eprintln!("404 render error: {err}");
            "404 Not Found".to_string()
        });

    (StatusCode::NOT_FOUND, Html(rendered))
}

async fn static_handler(Path(file): Path<String>) -> Response {
    let path = PathBuf::from(format!("static/{}", file));

    match fs::read(&path) {
        Ok(contents) => {
            let mime = if file.ends_with(".css") {
                "text/css"
            } else if file.ends_with(".js") {
                "application/javascript"
            } else if file.ends_with(".png") {
                "image/png"
            } else if file.ends_with(".jpg") || file.ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "application/octet-stream"
            };

            match Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(contents.into())
            {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Response build error: {e}");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Failed to build response".into())
                        .unwrap_or_else(|_| Response::new("Critical error".into()))
                }
            }
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("File not found".into())
            .unwrap_or_else(|_| Response::new("Critical error".into())),
    }
}
