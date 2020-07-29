use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Chapter {
    pub res: String,
    pub header: String,
    pub spans: Vec<String>,
}

impl Chapter {
    pub fn with_res(self, res: &str) -> Self {
        Self {
            res: res.to_string(),
            .. self
        }
    }
}
