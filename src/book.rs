use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::meta::MetaVar;

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
            ..self
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub chapters: Vec<Chapter>,
    pub meta: HashMap<String, MetaVar>,
}
