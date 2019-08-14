use super::Error;
use hyper::{Body, Client, Request, Response};
use log::{error, info};
use serde::{de::DeserializeOwned, Deserialize};

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

async fn request_body_into_string(body: Response<Body>) -> Result<Vec<u8>, Error> {
    let mut body = body.into_body();
    let mut out = Vec::new();
    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        out.extend_from_slice(chunk.as_ref());
    }
    Ok(out)
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
    let https = hyper_tls::HttpsConnector::new(1)?;
    let client = Client::builder().build::<_, Body>(https);

    let mut req: Request<_> = Request::new(Body::empty());
    req.headers_mut()
        .append("User-Agent", "GitHub StarCounter.rs".parse()?);
    if let Some(auth) = auth {
        req.headers_mut()
            .append("Authorization", auth.to_string().parse()?);
    }
    let url = format!("https://api.github.com/{}", url);
    *req.uri_mut() = url.parse().expect("valid URL");

    info!("{} - requested", url);
    let started = std::time::Instant::now();
    let res = client.request(req).await?;
    info!("{} - header received in {:?}", url, started.elapsed());

    let status = res.status();
    let started = std::time::Instant::now();
    let bytes = request_body_into_string(res).await?;
    info!("{} - body received in {:?}", url, started.elapsed());

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
