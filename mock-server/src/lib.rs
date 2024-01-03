use bytes::Bytes;
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::{oneshot, oneshot::Sender};
use warp::{reply, Filter, Rejection, Reply};

#[derive(Debug)]
pub struct MockServer {
    addr: SocketAddr,
}

#[derive(Debug)]
pub struct Handler {
    pub sender: Sender<i32>,
}

impl MockServer {
    pub fn new(addr: SocketAddr) -> MockServer {
        MockServer { addr }
    }

    async fn check_profanity(_: (), b: Bytes) -> Result<impl Reply, Rejection> {
        let content = String::from_utf8(b.to_vec()).expect("Invalid body");

        println!("=====> {:?}", content);

        if content.contains("shitty") {
            return Ok(reply::json(&json!({
              "bad_words_list": [
                {
                  "deviations": 0,
                  "end": 16,
                  "info": 2,
                  "original": "shitty",
                  "replacedLen": 6,
                  "start": 10,
                  "word": "shitty"
                }
              ],
              "bad_words_total": 1,
              "censored_content": "this is a ****** sentence",
              "content": "this is a shitty sentence"
            })))
        }

        Ok(reply::json(&json!({
          "bad_words_list": [
            {
              "deviations": 0,
              "end": 16,
              "info": 2,
              "original": "shitty",
              "replacedLen": 6,
              "start": 10,
              "word": "shitty"
            }
          ],
          "bad_words_total": 1,
          "censored_content": "this is a normal sentence",
          "content": "this is a normal sentence"
        })))
    }

    #[allow(opaque_hidden_inferred_bound)]
    fn routes(&self) -> impl Filter<Extract = impl Reply> + Clone {
        let bad_word_check = warp::post()
            .and(warp::path("bad_words"))
            .and(warp::query())
            .map(|_: HashMap<String, String>| ())
            .and(warp::path::end())
            .and(warp::body::bytes())
            .and_then(Self::check_profanity);

        bad_word_check
    }

    pub fn server(&self) -> Handler {
        let (tx, rx) = oneshot::channel::<i32>();

        let (_, server) =
            warp::serve(self.routes()).bind_with_graceful_shutdown(self.addr, async {
                rx.await.ok();
            });

        tokio::spawn(server);

        Handler { sender: tx }
    }
}
