use aws_sdk_s3::presigning::PresigningConfig;
use serde::Serialize;
use crate::{MAX_FILES, OBJECT_EXPIRATION_TIME};

#[derive(Debug, Serialize)]
struct GeneratePresignedUrlsResponse {
    pub presigned_urls: Vec<String>,
}

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
