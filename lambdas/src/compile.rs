use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

mod common;
use crate::common::{Item, Status};

const QUEUE_URL_DEFAULT: &str = "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";

#[derive(Debug, Deserialize)]
struct Request {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct Response {}

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
) {
    info!("Intitating compilation");

    // TODO: check s3 for files(?)
    let result = dynamo_client
        .get_item()
        .key("ID", AttributeValue::S(id.clone()))
        .send()
        .await
        .unwrap();
    if let Some(_) = result.item {
        return;
    }

    let item = Item {
        id: id.clone(),
        status: Status::Pending,
    };

    let response = dynamo_client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item.into()))
        .send()
        .await
        .unwrap();
    info!("db put response: {:?}", response);

    let message_output = sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(id)
        .send()
        .await
        .unwrap();

    info!(
        "message sent to sqs: {}",
        message_output.message_id.unwrap_or("empty_id".into())
    );
}

#[tracing::instrument(skip(event), fields(req_id = %event.context.request_id))]
async fn process_event(
    event: LambdaEvent<Request>,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
) -> Response {
    compile(
        event.payload.id,
        dynamo_client,
        table_name,
        sqs_client,
        queue_url,
    )
    .await;

    Response {}
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

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&config);
    let sqs_client = aws_sdk_sqs::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        Result::<_, Error>::Ok(
            process_event(event, &dynamo_client, &table_name, &sqs_client, &queue_url).await,
        )
    }))
    .await
}
