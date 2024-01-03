use std::collections::HashMap;

use error_handler::AppError;
use tracing::{error, info};
use warp::{http::StatusCode, reject, reply, Rejection, Reply};

use crate::store::Store;

pub async fn add_a(store: Store, body: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    info!("{:?}", body);
    let id = match get_qid(&body) {
        Err(_e) => return Err(reject::custom(AppError::QuestionNotFound)),
        Ok(id) => id,
    };

    match store.add_a(id, body.get("content").unwrap().clone()).await {
        Ok(_) => Ok(reply::with_status("Added", StatusCode::CREATED)),
        Err(e) => {
            error!("Failed to add ans: {:?}", e);
            Err(reject::custom(AppError::DbQueryError))
        }
    }
}

fn get_qid(body: &HashMap<String, String>) -> std::io::Result<i32> {
    let id = body.get("qid").ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Qid not found",
    ))?;
    let id = id
        .parse::<i32>()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid qid"))?;
    Ok(id)
}
