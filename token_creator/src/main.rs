use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use tokio::sync::Mutex;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let state = AppState {
        user_tokens: Arc::new(Mutex::new(HashMap::new())),
    };
    // build our application with a route
    let app = Router::new()
        .route("/token/:user_id", get(handler))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path(_user_id): Path<u32>, State(state): State<AppState>) -> String {
    let mut lock = state.user_tokens.lock_owned().await;
    if lock.is_empty() {
        let id = Uuid::new_v4().to_string();
        lock.insert(_user_id, id.clone());
        return id;
    } else {
        let existing_value = lock.get(&_user_id);
        if existing_value.is_some() {
            return existing_value.unwrap().clone();
        }
        let id = Uuid::new_v4().to_string();
        lock.insert(_user_id, id.clone());
        return id;
    }
}

#[derive(Clone)]
struct AppState {
    user_tokens: Arc<Mutex<HashMap<u32, String>>>,
}
