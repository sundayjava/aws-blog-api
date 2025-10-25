use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CreatePost {
    title: String,
    content: String,
    author: String,
}

type SharedState = Arc<RwLock<Vec<Post>>>;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting Blog API server...");

    // Create shared state
    let posts: SharedState = Arc::new(RwLock::new(vec![Post {
        id: 1,
        title: "Welcome to Rust".to_string(),
        content: "Learning Rust is awesome!".to_string(),
        author: "Admin".to_string(),
    }]));

    // Build router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/posts", get(get_posts).post(create_post))
        .route("/posts/{id}", get(get_post))
        .layer(CorsLayer::permissive())
        .with_state(posts);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[instrument]
async fn health_check() -> &'static str {
    info!("Health check requested");
    "OK"
}

#[instrument(skip(state))]
async fn get_posts(State(state): State<SharedState>) -> Json<Vec<Post>> {
    let posts = state.read().await;
    info!("Fetched {} posts", posts.len());
    Json(posts.clone())
}

#[instrument(skip(state))]
async fn get_post(
    Path(id): Path<u32>,
    State(state): State<SharedState>,
) -> Result<Json<Post>, StatusCode> {
    let posts = state.read().await;

    posts
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            info!("Post {} not found", id);
            StatusCode::NOT_FOUND
        })
}

#[instrument(skip(state))]
async fn create_post(
    State(state): State<SharedState>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let mut posts = state.write().await;

    let new_id = posts.iter().map(|p| p.id).max().unwrap_or(0) + 1;

    let new_post = Post {
        id: new_id,
        title: payload.title,
        content: payload.content,
        author: payload.author,
    };

    info!("Created post with id: {}", new_id);
    posts.push(new_post.clone());

    Ok(Json(new_post))
}
