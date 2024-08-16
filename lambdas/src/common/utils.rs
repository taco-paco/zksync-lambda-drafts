use lambda_http::{Error, Request, RequestPayloadExt, Response};
use serde::{Deserialize, Deserializer};
use serde::de::DeserializeOwned;

const EMPTY_PAYLOAD_ERROR: &str = "Request payload is empty";

pub fn extract_request<T: DeserializeOwned>(
    request: Request,
) -> Result<Result<T, Response<String>>, Error> {
    return match request.payload::<T>() {
        Ok(Some(val)) => Ok(Ok(val)),
        Ok(None) => {
            let response = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(EMPTY_PAYLOAD_ERROR.into())
                .map_err(Box::new)?;

            return Ok(Err(response));
        }
        Err(err) => {
            let response = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;

            Ok(Err(response))
        }
    };
}
