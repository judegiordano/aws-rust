pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Message {
        pub message: String,
    }
}
