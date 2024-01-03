use chrono::prelude::*;
use chrono::Duration;
use error_handler::AppError;
use paseto::PasetoBuilder;
use std::future;
use tracing::{error, info};
use warp::http::StatusCode;
use warp::{reject, reply, Filter, Rejection, Reply};

use crate::types::account::{AccountId, Session};
use crate::utils::{check_password, hash_password};
use crate::{store::Store, types::account::Account};

const SECRET: &str = "RANDOM WORDS WINTER MACINTOSH PC";

pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    let hashed = hash_password(account.password);
    let account = Account {
        id: None,
        email: account.email,
        password: hashed,
    };

    info!("Register new user {:#?}", account);

    match store.add_account(account).await {
        Ok(_) => Ok(reply::with_status("Created", StatusCode::CREATED)),
        Err(e) => {
            error!("Failed to add account: {e:#?}");
            Err(reject::custom(AppError::DbQueryError))
        }
    }
}

pub async fn login(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    let store_acc = match store.find_account(account.email).await {
        Err(_e) => return Err(reject::custom(AppError::DbQueryError)),
        Ok(a) => a,
    };

    if check_password(account.password, store_acc.password) {
        Ok(reply::with_header(
            "",
            "Authorization",
            issue_token(store_acc.id.unwrap()),
        ))
    } else {
        Err(reject::custom(AppError::InvalidCredential))
    }
}

fn issue_token(id: AccountId) -> String {
    let now = Utc::now();
    let exp = now + Duration::days(1);
    // let state = serde_json::to_string(&id).expect("Failed to serialize");
    PasetoBuilder::new()
        .set_encryption_key(SECRET.as_bytes())
        .set_expiration(&exp)
        .set_not_before(&now)
        .set_claim("id", serde_json::json!(id))
        .build()
        .expect("Failed to create token")
    // local_paseto(&state, None, b"RANDOM WORDS WINTER MACINTOSH PC").expect("Failed to create token")
}

fn verify_token(token: String) -> Result<Session, AppError> {
    let json = paseto::tokens::validate_local_token(
        token.as_str(),
        None,
        SECRET.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|e| {
        error!("Failed to decrypt token: {:?}", e);
        AppError::InvalidToken
    })?;

    serde_json::from_value::<Session>(json).map_err(|e| {
        error!("Failed to parse token: {:?}", e);
        AppError::InvalidToken
    })
}

pub type One<T> = (T,);

pub(crate) fn auth() -> impl Filter<Extract = One<Session>, Error = Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let session = match verify_token(token) {
            Ok(session) => session,
            Err(_) => return future::ready(Err(warp::reject::custom(AppError::InvalidToken))),
        };
        future::ready(Ok(session))
    })
}

#[cfg(test)]
mod auth_tests {
    use error_handler::AppError;

    use crate::{
        routes::auth::{auth, issue_token},
        types::account::AccountId,
    };

    #[tokio::test]
    async fn test_auth() {
        let token = issue_token(AccountId(2));
        let filter = auth();
        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter)
            .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().id.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_auth_error() {
        let filter = auth();
        let res = warp::test::request()
            .header("Authorization", "token")
            .filter(&filter)
            .await;

        assert!(res.is_err());

        let rej = res.unwrap_err();
        assert_eq!(
            rej.find::<AppError>().unwrap().to_string(),
            AppError::InvalidToken.to_string()
        );
    }
}
