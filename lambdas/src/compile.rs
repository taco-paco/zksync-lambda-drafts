use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{
    run, service_fn, Error, LambdaEvent, Request as LambdaRequest, Response as LambdaResponse,
};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{error, info};

mod common;
use crate::common::utils::extract_request;
use crate::common::{Item, Status, BUCKET_NAME_DEFAULT};

const QUEUE_URL_DEFAULT: &str = "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";

const NO_OBJECTS_TO_COMPILE_ERROR: &str = "There are no objects to compile";

#[derive(Debug, Deserialize)]
struct Request {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct Response {}

// impl Deserialize for Response {
//     fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
//         todo!()
//     }
// }
// TODO:
// struct SqsClient {
//     pub client: aws_sdk_sqs::Client,
//     pub queue_url: String,
//    // pub other_data: String
// }

// TODO: maybe needs s3_client to check that files uploaded. Or fail on EC2 side?
async fn compile(
    id: String,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
) -> Result<Result<(), LambdaResponse<String>>, Error> {
    info!("Check if same id exists in db");
    let result = dynamo_client
        .get_item()
        .key("ID", AttributeValue::S(id.clone()))
        .send()
        .await
        .unwrap();

    if let Some(_) = result.item {
        error!("Recompilation attempt");

        let response = lambda_http::Response::builder()
            .status(400)
            .header("content-type", "text/html")
            .body("Recompilation attempt".into())
            .map_err(Box::new)?;

        return Ok(Err(response));
    }

    let item = Item {
        id: id.clone(),
        status: Status::Pending,
    };

    {
        info!("Initializing item with id: {}", id.clone());
        let response = dynamo_client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item.into()))
            .send()
            .await
            .map_err(Box::new)?;
        info!("Initialized: {:?}", response);
    }

    info!("Sending message to SQS");
    let message_output = sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(id)
        .send()
        .await
        .map_err(Box::new)?;

    info!(
        "message sent to sqs: {}",
        message_output.message_id.unwrap_or("empty_id".into())
    );

    Ok(Ok(()))
}

#[tracing::instrument]
async fn process_request(
    request: LambdaRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
    s3_client: &aws_sdk_s3::Client,
    bucket_name: &str,
) -> Result<LambdaResponse<String>, Error> {
    let request = extract_request::<Request>(request)??;

    info!("Checking if objects in folder");
    let objects = s3_client
        .list_objects_v2()
        .delimiter('/')
        .prefix(request.id.clone())
        .bucket(bucket_name)
        .send()
        .await
        .map_err(Box::new)?;

    if let None = objects.contents {
        error!("No objects in folder");
        let response = LambdaResponse::builder()
            .status(400)
            .header("content-type", "text/html")
            .body(NO_OBJECTS_TO_COMPILE_ERROR.into())
            .map_err(Box::new)?;

        return Ok(response);
    } else {
        info!("objects num: {}", objects.contents.unwrap().len());
    }

    info!("Intitializing compilation");
    compile(request.id, dynamo_client, table_name, sqs_client, queue_url).await?;

    let response = LambdaResponse::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Default::default())
        .map_err(Box::new)?;

    return Ok(response);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time() // CloudWatch will add the ingestion time
        .with_target(false)
        .init();

    let queue_url = std::env::var("QUEUE_URL").unwrap_or(QUEUE_URL_DEFAULT.into());
    let table_name = std::env::var("TABLE_NAME").unwrap_or(TABLE_NAME_DEFAULT.into());
    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or(BUCKET_NAME_DEFAULT.into());

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&config);
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let s3_client = aws_sdk_s3::Client::new(&config);

    run(service_fn(|request: LambdaRequest| async {
        process_request(
            request,
            &dynamo_client,
            &table_name,
            &sqs_client,
            &queue_url,
            &s3_client,
            &bucket_name,
        )
        .await
    }))
    .await
}
