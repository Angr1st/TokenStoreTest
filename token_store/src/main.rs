use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use token_common::Token;
use tokio::sync::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/token/{user_id}")]
async fn request_user_token(
    state: web::Data<AppState>,
    path: web::Path<u32>,
) -> Result<impl Responder> {
    let mut user_tokens = state.user_tokens.lock().await;
    if user_tokens.contains_key(&path) {
        let current_token = user_tokens.get(&path).expect("Value should be there");
        let now = Utc::now();
        let expiry_date = current_token.compute_expiry_date();
        if now < expiry_date {
            return Ok(web::Json(current_token.clone()));
        }
    }

    let resp = reqwest::get(format!("http://localhost:3000/token/{}", &path))
        .await
        .unwrap()
        .json::<Token>()
        .await
        .unwrap();
    user_tokens.insert(*path, resp.clone());
    Ok(web::Json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppState {
        user_tokens: Arc::new(Mutex::new(HashMap::new())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(request_user_token)
            .service(hello)
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

struct AppState {
    user_tokens: Arc<Mutex<HashMap<u32, Token>>>,
}
