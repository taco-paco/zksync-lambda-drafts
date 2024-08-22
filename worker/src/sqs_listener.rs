use crate::errors::{DeleteError, ReceiveError};
use async_channel::{Receiver, Recv, Sender};
use aws_sdk_sqs::config::http::HttpResponse;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageError;
use aws_sdk_sqs::types::Message;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::sqs_client::SqsClient;

#[derive(Clone)]
pub struct SqsListener {
    receiver: Receiver<Message>,
    client: SqsClient,
}

impl SqsListener {
    pub fn new(
        client: SqsClient,
        poll_interval: Duration,
    ) -> (Self, JoinHandle<Result<(), ReceiveError>>) {
        // TODO: unbounded?
        let (sender, receiver) = async_channel::bounded(1000);
        let handle = tokio::spawn(Self::listen(client.clone(), sender, poll_interval));
        // TODO: close?
        // let (done_notifier, done_receiver) = tokio::sync::oneshot::channel();

        (Self { receiver, client }, handle)
    }

    async fn listen(
        client: SqsClient,
        sender: Sender<Message>,
        poll_interval: Duration,
    ) -> Result<(), SdkError<ReceiveMessageError, HttpResponse>> {
        loop {
            let response = client.receive_message().await?;
            let messages = if let Some(messages) = response.messages {
                messages
            } else {
                continue;
            };

            for message in messages {
                if sender.send(message).await.is_err() {
                    return Ok(());
                }
            }

            sleep(poll_interval).await;
        }
    }

    pub fn recv(&self) -> Recv<'_, Message> {
        self.receiver.recv()
    }

    pub async fn delete_message(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<(), DeleteError> {
        self.client.delete_message(receipt_handle).await
    }

    // TODO: done/close
}

struct RunningSqsListener {
    join_handle: JoinHandle<Result<(), ReceiveError>>,
    sqs_listener: SqsListener,
}

impl RunningSqsListener {
    pub fn new(
        join_handle: JoinHandle<Result<(), ReceiveError>>,
        sqs_listener: SqsListener,
    ) -> Self {
        Self {
            join_handle,
            sqs_listener,
        }
    }

    pub fn sqs_listener(&self) -> &SqsListener {
        &self.sqs_listener
    }
}
