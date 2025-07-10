use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tera::{Context, Tera};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let tera = Arc::new(Mutex::new(
        Tera::new("templates/**/*").expect("Failed to load templates"),
    ));

    let app = Router::new()
        .route("/{name}", get(name))
        .fallback(handler_404)
        .with_state(tera.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn name(Path(name): Path<String>, State(tera): State<Arc<Mutex<Tera>>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("name", &name);

    let tera = tera.lock().await;
    let rendered = tera
        .render("name.html", &context)
        .expect("Failed to render template");

    Html(rendered)
}

async fn handler_404(State(tera): State<Arc<Mutex<Tera>>>) -> impl IntoResponse {
    let tera = tera.lock().await;
    let rendered = tera
        .render("404.html", &Context::new())
        .unwrap_or_else(|_| "404 Not Found".to_string());

    (axum::http::StatusCode::NOT_FOUND, Html(rendered))
}
