use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Deserialize)]
struct InvokeRequest {
    function: String,
    args: Vec<String>,
}

#[derive(Serialize)]
struct InvokeResponse {
    result: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/deploy", post(deploy_contract))
        .route("/invoke", post(invoke_contract))
        .route("/api", get(get_api))
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn deploy_contract() -> Result<Json<serde_json::Value>, StatusCode> {
    // Run the deploy script for local network
    let output = Command::new("bash")
        .arg("../../scripts/deploy-local.sh")
        .arg("--non-interactive")
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Extract contract ID from output
        let contract_id = extract_contract_id(&stdout);
        // Save to file
        fs::write("contract_id.txt", contract_id).ok();
        Ok(Json(serde_json::json!({ "contract_id": contract_id })))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

fn extract_contract_id(output: &str) -> &str {
    // Simple extraction, assuming the last line has "Contract ID: <id>"
    output.lines().last().unwrap_or("").split(": ").nth(1).unwrap_or("")
}

async fn invoke_contract(
    Json(req): Json<InvokeRequest>,
) -> Result<Json<InvokeResponse>, StatusCode> {
    // Read contract ID
    let contract_id = fs::read_to_string("contract_id.txt").map_err(|_| StatusCode::BAD_REQUEST)?;

    // Use soroban contract invoke
    let mut cmd = Command::new("soroban");
    cmd.arg("contract").arg("invoke")
        .arg("--id").arg(contract_id.trim())
        .arg("--source").arg("teachlink-deployer")
        .arg("--network").arg("local")
        .arg("--")
        .arg(&req.function);

    for arg in &req.args {
        cmd.arg(arg);
    }

    let output = cmd.output().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Json(InvokeResponse { result }))
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(Json(InvokeResponse { result: format!("Error: {}", error) }))
    }
}

async fn get_api() -> Json<serde_json::Value> {
    // Return the API spec
    Json(serde_json::json!({
        "functions": [
            {
                "name": "initialize",
                "args": ["token:Address", "admin:Address", "min_validators:u32", "fee_recipient:Address"]
            },
            {
                "name": "bridge_out",
                "args": ["from:Address", "amount:i128", "destination_chain:u32", "destination_address:Bytes"]
            },
            {
                "name": "complete_bridge",
                "args": ["message:CrossChainMessage", "validator_signatures:Vec<Address>"]
            },
            {
                "name": "add_validator",
                "args": ["validator:Address"]
            },
            {
                "name": "initialize_rewards",
                "args": ["token:Address", "rewards_admin:Address"]
            },
            {
                "name": "create_escrow",
                "args": ["depositor:Address", "beneficiary:Address", "token:Address", "amount:i128", "signers:Vec<Address>", "threshold:u32", "release_time:Option<u64>", "refund_time:Option<u64>", "arbitrator:Address"]
            },
            // Add more as needed
        ]
    }))
}