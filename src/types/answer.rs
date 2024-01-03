use std::{io::Error, str::FromStr};
use super::question::QuestionId;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct AnswerId(String);

impl FromStr for AnswerId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AnswerId(s.to_string()))
    }
}

#[derive(Debug)]
pub struct Answer {
    pub id: AnswerId,
    pub qid: QuestionId,
    pub content: String,
}
