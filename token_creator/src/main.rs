use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
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

async fn handler(Path(_user_id): Path<u32>, State(state): State<AppState>) -> Json<Token> {
    let mut lock = state.user_tokens.lock_owned().await;
    if lock.is_empty() {
        let id = Uuid::new_v4().to_string();
        let token = Token::new(_user_id, id.clone());
        lock.insert(_user_id, token.clone());
        return Json(token);
    } else {
        let existing_value = lock.get(&_user_id);
        match existing_value {
            Some(value) => {
                let now = Utc::now();
                let expiry_date = value.compute_expiry_date();
                if expiry_date > now {
                    return Json(value.clone());
                } else {
                    let id = Uuid::new_v4().to_string();
                    let token = Token::new(_user_id, id.clone());
                    lock.insert(_user_id, token.clone());
                    return Json(token);
                }
            }
            None => {
                let id = Uuid::new_v4().to_string();
                let token = Token::new(_user_id, id.clone());
                lock.insert(_user_id, token.clone());
                return Json(token);
            }
        }
    }
}

#[derive(Clone)]
struct AppState {
    user_tokens: Arc<Mutex<HashMap<u32, Token>>>,
}

#[derive(Serialize, Clone)]
struct Token {
    created: DateTime<Utc>,
    expires_in: u64,
    user_id: u32,
    value: String,
}

impl Token {
    fn new(user_id: u32, token_value: String) -> Self {
        Self {
            created: Utc::now(),
            expires_in: 3600,
            user_id,
            value: token_value,
        }
    }

    fn expiration_duration(&self) -> chrono::Duration {
        chrono::Duration::seconds(
            self.expires_in
                .try_into()
                .expect("3600 seconds should always work."),
        )
    }

    fn compute_expiry_date(&self) -> DateTime<Utc> {
        let expiration_duration = self.expiration_duration();
        self.created
            .checked_add_signed(expiration_duration)
            .expect("This should always return a valid DateTime!")
    }
}
