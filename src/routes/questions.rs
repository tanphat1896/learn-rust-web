use std::collections::HashMap;

use crate::{
    store::Store,
    types::{
        paging::{extract_paging, Pagination},
        question::QuestionPayload, account::Session,
    },
};
use error_handler::AppError;
use tracing::{error, info};
use warp::{http::StatusCode, reject, reply, Rejection, Reply};

use crate::profanity::check_profanity;

// #[instrument]
pub async fn get_q(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    // store.track();

    // thread::sleep(Duration::from_secs(1));
    info!("Get list q");

    let paging = if params.is_empty() {
        info!(paging = false);
        Pagination::default()
    } else {
        info!(paging = true);
        extract_paging(params)?
    };

    let res = store.get_q(paging.limit, paging.offset).await;
    match res {
        Ok(qs) => Ok(reply::json(&qs)),
        Err(e) => {
            error!("Failed to get list of questions {:?}", e);
            Err(warp::reject::custom(AppError::DbQueryError))
        }
    }
}

pub async fn add_q(s: Session, store: Store, q: QuestionPayload) -> Result<impl Reply, Rejection> {
    let title = tokio::spawn(check_profanity(q.title));
    let content = tokio::spawn(check_profanity(q.content));

    println!("{s:#?}");
    

    let (title, content) = (title.await.unwrap(), content.await.unwrap());

    if title.is_err() {
        return Err(reject::custom(title.unwrap_err()));
    }

    if content.is_err() {
        return Err(reject::custom(content.unwrap_err()));
    }

    match store
        .add_q(QuestionPayload {
            title: title.unwrap().censored_content,
            content: content.unwrap().censored_content,
            tags: q.tags,
        })
        .await
    {
        Ok(q) => Ok(reply::with_status(reply::json(&q), StatusCode::CREATED)),
        Err(e) => {
            error!("Failed to add question {:?}", e);
            Err(reject::custom(AppError::DbQueryError))
        }
    }
}

pub async fn detail_q(_id: u32, _store: Store) -> Result<impl Reply, Rejection> {
    // match store.questions.read().await.get(&QuestionId(id)) {
    //     Some(q) => Ok(reply::json(&q)),
    //     None => Err(reject::custom(AppError::QuestionNotFound)),
    // }
    Ok(reply::with_status("Created", StatusCode::CREATED))
}

pub async fn upd_q(id: u32, store: Store, q: QuestionPayload) -> Result<impl Reply, Rejection> {
    match store.upd_q(id as i32, q).await {
        Ok(id) => Ok(reply::with_status(
            format!("Updated {id}"),
            StatusCode::ACCEPTED,
        )),
        Err(e) => {
            error!("Failed to update question {:?}", e);
            Err(warp::reject::custom(AppError::DbQueryError))
        }
    }
}

pub async fn del_q(id: u32, store: Store) -> Result<impl Reply, Rejection> {
    match store.del_q(id as i32).await {
        Ok(id) => Ok(reply::with_status(
            format!("{id} has been deleted"),
            StatusCode::ACCEPTED,
        )),
        Err(e) => {
            error!("Failed to delete question {:?}", e);
            Err(warp::reject::custom(AppError::DbQueryError))
        }
    }
}
