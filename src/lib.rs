pub mod types {
    use lambda_http::{http::StatusCode, Response};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::fmt::Debug;

    pub trait ResponseHelper: Serialize {
        fn to_response(&self) -> anyhow::Result<Response<String>> {
            let body = json!(self).to_string();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(body)?;
            Ok(response)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Message {
        pub message: String,
    }

    impl ResponseHelper for Message {}
}
