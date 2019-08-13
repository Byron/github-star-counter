use super::Error;
use hyper::{Body, Client, Request, Response};
use serde::{de::DeserializeOwned, Deserialize};

pub struct BasicAuth {
    username: String,
    password: Option<String>,
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

pub async fn json<D>(url: &str, auth: Option<BasicAuth>) -> Result<D, Error>
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
    *req.uri_mut() = format!("https://api.github.com/{}", url)
        .parse()
        .expect("valid URL");

    let res = client.request(req).await?;
    let status = res.status();
    let bytes = request_body_into_string(res).await?;

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
