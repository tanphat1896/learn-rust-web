use std::{
    env,
    time::{Duration, Instant},
};

use error_handler::AppError;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadWordResponse {
    pub bad_words_list: Vec<BadWordsList>,
    pub bad_words_total: i64,
    pub censored_content: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadWordsList {
    pub original: String,
    pub word: String,
    #[serde(rename = "replacedLen")]
    pub replaced_len: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadWordErrorRes {
    pub message: String,
}

pub async fn check_profanity(text: String) -> Result<BadWordResponse, AppError> {
    info!("Starting check text: {}", &text);
    let start = Instant::now();
    let rs = api_layer(text.clone()).await;
    info!(
        "Finish check text: {}, consumes: {}ms",
        &text,
        start.elapsed().as_millis()
    );
    rs
}

#[instrument]
async fn api_layer(text: String) -> Result<BadWordResponse, AppError> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let api_key = option_env!("API_LAYER_K").map_or("", |k| k);
    let api_url =
        env::var("API_LAYER_URL").map_or("https://api.apilayer.com".to_string(), |url| url);

    let endpoint = format!("{}/bad_words?censor_character=*", api_url);
    println!("Will connect to {:?}", endpoint);

    let res = client
        .post(endpoint)
        .timeout(Duration::from_secs(3))
        .header("apikey", api_key)
        .body(text)
        .send()
        .await
        .map_err(|e| AppError::ApiCallErr(e.to_string()))?;

    let status = res.status();

    if !status.is_success() {
        let error = res
            .json::<BadWordErrorRes>()
            .await
            .map_err(|e| AppError::ApiCallErr(e.to_string()))?;
        return Err(AppError::ApiCallErr(error.message));
    }

    let data = res
        .json::<BadWordResponse>()
        .await
        .map_err(|e| AppError::ApiCallErr(e.to_string()))?;

    Ok(data)
}

#[cfg(test)]
mod profanity_tests {
    use std::{env, net::SocketAddr};

    use super::check_profanity;

    fn setup() -> mock_server::Handler {
        let addr = "127.0.0.1:3333".parse::<SocketAddr>().unwrap();
        env::set_var("API_LAYER_URL", "http://localhost:3333");
        let server = mock_server::MockServer::new(addr);
        server.server()
    }

    #[tokio::test]
    async fn api_test() {
        let handler = setup();
        normal_content().await;
        profane_content().await;
        let _ = handler.sender.send(1);
    }

    async fn normal_content() {
        let rs = check_profanity("fafsa".to_string()).await;
        assert!(rs.is_ok());
        assert_eq!(rs.unwrap().censored_content, "this is a normal sentence");
    }

    async fn profane_content() {
        let rs = check_profanity("shitty".to_string()).await;
        assert!(rs.is_ok());
        assert_eq!(rs.unwrap().censored_content, "this is a ****** sentence");
    }
}
