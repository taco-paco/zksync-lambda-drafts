use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod common;
use common::BUCKET_NAME_DEFAULT;

const MAX_FILES: usize = 300;
const OBJECT_EXPIRATION_TIME: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Deserialize)]
struct Request {
    pub files: Vec<String>,
    // TODO: add files_md5. Use set_content_md5 on object
}

#[derive(Debug, Serialize)]
struct Response {
    pub presigned_urls: Vec<String>,
}

async fn generate_presigned_urs(
    files: Vec<String>,
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
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

async fn process_event(
    request: Request,
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
) -> Response {
    let presigned_urls = generate_presigned_urs(request.files, bucket_name, s3_client).await;
    Response { presigned_urls }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("generate_presigned_urls");

    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or(BUCKET_NAME_DEFAULT.into());

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        Result::<_, Error>::Ok(process_event(event.payload, &bucket_name, &s3_client).await)
    }))
    .await
}
