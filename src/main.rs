mod compile;
mod generate_presigned_urls;

use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BUCKET_NAME_DEFAULT: &str = "zksync-compilation";
const OBJECT_EXPIRATION_TIME: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_FILES: usize = 300;

/// Request types
#[derive(Debug, Deserialize)]
struct GeneratePresignedUrlsRequest {
    pub files: Vec<String>,
    // TODO: add files_md5. Use set_content_md5 on object
}

#[derive(Debug,, Deserialize)]
struct CompileRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct PollRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
enum Request {
    GeneratePresignedUrls(GeneratePresignedUrlsRequest),
    Compile(CompileRequest),
    Poll(PollRequest),
}

/// Response types
#[derive(Debug, Serialize)]
struct GeneratePresignedUrlsResponse {
    pub presigned_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CompileResponse {
    pub presigned_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
enum Status {
    Pending,
    Compiling,
    Ready(String),
    Failed(String),
}

#[derive(Debug, Serialize)]
struct PollResponse {
    pub status: Status,
}

#[derive(Debug, Serialize)]
enum Response {
    GeneratePresignedUrls(GeneratePresignedUrlsResponse),
    Compile(CompileResponse),
    Poll(PollResponse),
}

/// Lambda functions
async fn generate_presigned_urs(
    files: Vec<String>,
    bucket_name: &str,
    s3_client: aws_sdk_s3::Client,
) -> Vec<String> {
    if files.len() > MAX_FILES {
        // TODO: throw error
        todo!()
    }

    let mut output = Vec::with_capacity(files.len());
    for file in files {
        let presigned = s3_client
            .put_object()
            .bucket(bucket_name)
            .key(file)
            .presigned(PresigningConfig::expires_in(OBJECT_EXPIRATION_TIME).unwrap())
            .await
            .unwrap();

        // TODO: logging
        println!("{:?}", presigned);

        output.push(presigned.uri().into());
    }

    output
}

// TODO: maybe needs s3_client to check that files uploaded. Or fail on EC2 side?
fn compile(id: String, dynamo_client: aws_sdk_dynamodb::Client) {}

fn poll(id: String, dynamo_client: aws_sdk_dynamodb::Client) {}

async fn process_event(
    event: LambdaEvent<Request>,
    s3_client: aws_sdk_s3::Client,
    bucket_name: &str,
    dynamo_client: aws_sdk_dynamodb::Client,
) {
    match event.payload {
        Request::GeneratePresignedUrls(request) => {
            generate_presigned_urs(request.files, bucket_name, s3_client);
        }
        Request::Compile(request) => compile(request.id, dynamo_client),
        Request::Poll(request) => poll(request.id, dynamo_client),
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or(BUCKET_NAME_DEFAULT.into());

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        process_event(event, s3_client, &bucket_name, dynamo_client).await
    }))
}
