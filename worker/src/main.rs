mod errors;
mod sqs_client;
mod sqs_listener;

use crate::sqs_client::SqsClient;
use crate::sqs_listener::SqsListener;
use aws_config::BehaviorVersion;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

const AWS_PROFILE_DEFAULT: &str = "dev";
// TODO: remove
pub(crate) const QUEUE_URL_DEFAULT: &str =
    "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";

#[tokio::main]
async fn main() {
    let profile_name = std::env::var("AWS_PROFILE").unwrap_or(AWS_PROFILE_DEFAULT.into());
    let profile_files = EnvConfigFiles::builder()
        .with_file(EnvConfigFileKind::Credentials, "./credentials")
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .profile_files(profile_files)
        .profile_name(profile_name)
        .region("ap-southeast-2")
        .load()
        .await;

    // Initialize SQS client
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let sqs_client = SqsClient::new(sqs_client, QUEUE_URL_DEFAULT);
    // // Example: Send a message to an SQS queue
    // let send_result = sqs_client
    //     .send_message()
    //     .queue_url(QUEUE_URL_DEFAULT)
    //     .message_body("Hello from Rust!")
    //     .send()
    //     .await
    //     .map_err(|err| println!("{}", err.to_string()))
    //     .expect("Oops");

    let (sqs_listener, handle) = SqsListener::new(sqs_client, Duration::from_secs(1));

    let handle = tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        1
    });

    let mut arc1 = Arc::new(handle);

    while let Ok(message) = sqs_listener.recv().await {
        println!("{:?}", message);
        if let Some(receipt_handle) = message.receipt_handle {
            sqs_listener
                .delete_message(receipt_handle)
                .await
                .map_err(|err| println!("delete error: {}", err.to_string()))
                .unwrap();
        }
    }
}
