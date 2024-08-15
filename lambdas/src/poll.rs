use crate::common::Status;
use serde::{Deserialize, Serialize};

mod common;

#[derive(Debug, Deserialize)]
struct Request {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct Response {
    pub status: Status,
}

async fn poll(id: String, dynamo_client: aws_sdk_dynamodb::Client) {}

#[tokio::main]
async fn main() {}
