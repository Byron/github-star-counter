use super::Error;
use log::{error, info};
use serde::{de::DeserializeOwned, Deserialize};
use std::ops::AddAssign;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    pub static ref TOTAL_DURATION: Mutex<Duration> = Mutex::new(Duration::default());
    pub static ref TOTAL_BYTES_RECEIVED_IN_BODY: AtomicU64 = AtomicU64::default();
}

#[derive(Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

impl ToString for BasicAuth {
    fn to_string(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(&match &self.password {
                Some(password) => format!("{}:{}", self.username, password),
                None => self.username.clone(),
            })
        )
    }
}

pub async fn json_log_failure<D>(url: String, auth: Option<BasicAuth>) -> Result<D, Error>
// TODO want Result<impl DeserializeOwned, ...> but that does not compile
where
    D: DeserializeOwned,
{
    match json(url, auth).await {
        Ok(v) => Ok(v),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

// TODO: Can the url string also NOT be owned?
pub async fn json<D>(url: String, auth: Option<BasicAuth>) -> Result<D, Error>
// TODO want Result<impl DeserializeOwned, ...> but that does not compile
where
    D: DeserializeOwned,
{
    let url = format!("https://api.github.com/{}", url);
    let mut req = surf::get(&url);
    req = req.set_header("User-Agent", "GitHub StarCounter.rs");
    if let Some(auth) = auth {
        req = req.set_header("Authorization", auth.to_string());
    }
    info!("{} - requested", url);
    let started = std::time::Instant::now();
    let mut res = req.await.map_err(|e| e.to_string())?;
    let status = res.status();
    let bytes = res.body_bytes().await?;
    let elapsed = started.elapsed();
    info!(
        "{} - received in {:?} ({})",
        url,
        elapsed,
        bytesize::ByteSize(bytes.len() as u64)
    );
    TOTAL_DURATION.lock().unwrap().add_assign(elapsed);
    TOTAL_BYTES_RECEIVED_IN_BODY.fetch_add(bytes.len() as u64, Ordering::Relaxed);

    if status.is_success() {
        Ok(serde_json::from_slice(&bytes)?)
    } else {
        #[derive(Deserialize)]
        struct Error {
            message: String,
        }
        let err: Error = serde_json::from_slice(&bytes).or_else(|e| {
            Ok::<_, serde_json::Error>(Error {
                message: format!(
                    "Unexpected error message format returned by Github: '{:#?}'",
                    e
                ),
            })
        })?;
        Err(err.message.into())
    }
}
