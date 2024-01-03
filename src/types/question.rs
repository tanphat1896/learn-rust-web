use std::{str::FromStr, io::Error};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestionPayload {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone, Eq, Hash, PartialEq, Deserialize)]
pub struct QuestionId(pub u32);

impl FromStr for QuestionId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(QuestionId(s.parse::<u32>().unwrap()))
    }
}
