use axum::{
    extract::Query,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use soroban_sdk::{Env, Symbol};
use teachlink_contract::TeachLinkContract;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/run", get(run_example))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

#[derive(Deserialize)]
struct RunQuery {
    func: String,
    params: String,
}

async fn run_example(Query(params): Query<RunQuery>) -> Result<String, StatusCode> {
    let env = Env::default();
    let result = match params.func.as_str() {
        "hello" => {
            let input = Symbol::new(&env, &params.params);
            TeachLinkContract::hello(env, input)
        }
        "add" => {
            let parts: Vec<&str> = params.params.split(',').collect();
            if parts.len() == 2 {
                let a: u32 = parts[0].trim().parse().unwrap_or(0);
                let b: u32 = parts[1].trim().parse().unwrap_or(0);
                TeachLinkContract::add(env, a, b)
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    Ok(format!("Result: {:?}", result))
}