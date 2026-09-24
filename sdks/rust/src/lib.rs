use reqwest::blocking::{multipart, Client as HttpClient, RequestBuilder};
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Configuration(&'static str),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Formalini returned HTTP {status}: {message}")]
    Api {
        status: StatusCode,
        message: String,
        retry_after: Option<String>,
    },
    #[error("Formalini usage limit: {message}")]
    UsageLimit { message: String },
    #[error("Unexpected Formalini response")]
    MalformedResponse,
}

pub struct Client {
    http: HttpClient,
    api_key: String,
    base_url: Url,
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::with_base_url(
            api_key,
            "https://api.formalini.com/api/public/v1/",
            Duration::from_secs(60),
        )
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: &str,
        timeout: Duration,
    ) -> Result<Self, Error> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() {
            return Err(Error::Configuration("API key is required"));
        }
        let mut base_url =
            Url::parse(base_url).map_err(|_| Error::Configuration("Invalid base URL"))?;
        if base_url.scheme() != "https" && !(cfg!(test) && base_url.scheme() == "http") {
            return Err(Error::Configuration("Base URL must use HTTPS"));
        }
        if timeout.is_zero() {
            return Err(Error::Configuration("Timeout must be positive"));
        }
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let http = HttpClient::builder().timeout(timeout).build()?;
        Ok(Self {
            http,
            api_key,
            base_url,
        })
    }

    pub fn documents(&self) -> Documents<'_> {
        Documents { client: self }
    }
    pub fn workflows(&self) -> Workflows<'_> {
        Workflows { client: self }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> Result<RequestBuilder, Error> {
        let url = self
            .base_url
            .join(path)
            .map_err(|_| Error::Configuration("Invalid path"))?;
        Ok(self.http.request(method, url).bearer_auth(&self.api_key))
    }

    fn document_url(&self, id: &str) -> Result<Url, Error> {
        if id.is_empty() {
            return Err(Error::Configuration("Document ID is required"));
        }
        let mut url = self
            .base_url
            .join("documents/")
            .map_err(|_| Error::Configuration("Invalid path"))?;
        url.path_segments_mut()
            .map_err(|_| Error::Configuration("Invalid path"))?
            .pop_if_empty()
            .push(id);
        Ok(url)
    }

    fn send(&self, request: RequestBuilder) -> Result<Value, Error> {
        let response = request.send()?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|header| header.to_str().ok())
            .map(str::to_owned);
        let body: Value = match response.json() {
            Ok(body) => body,
            Err(_) if status.is_success() => return Err(Error::MalformedResponse),
            Err(_) if status == StatusCode::PAYMENT_REQUIRED => {
                return Err(Error::UsageLimit {
                    message: format!("Formalini returned HTTP {status}"),
                })
            }
            Err(_) => {
                return Err(Error::Api {
                    status,
                    message: format!("Formalini returned HTTP {status}"),
                    retry_after,
                })
            }
        };
        if !status.is_success() {
            let message = body
                .get("error")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Formalini returned HTTP {status}"));
            if status == StatusCode::PAYMENT_REQUIRED {
                return Err(Error::UsageLimit { message });
            }
            return Err(Error::Api {
                status,
                message,
                retry_after,
            });
        }
        if !body.is_object() {
            return Err(Error::MalformedResponse);
        }
        Ok(body)
    }
}

pub struct Documents<'a> {
    client: &'a Client,
}

#[derive(Serialize)]
pub struct CreateDocument<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_date: Option<&'a str>,
}

#[derive(Serialize)]
pub struct UpdateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Value>>,
}

pub struct Upload<'a> {
    pub pages: &'a [PathBuf],
    pub name: Option<&'a str>,
    pub template_id: Option<&'a str>,
    pub asynchronous: bool,
}

impl Documents<'_> {
    pub fn list(&self, limit: u8, status: Option<&str>) -> Result<Vec<Value>, Error> {
        if limit == 0 || limit > 100 {
            return Err(Error::Configuration("Limit must be between 1 and 100"));
        }
        if let Some(status) = status {
            if ![
                "queued",
                "processing",
                "review",
                "approved",
                "exported",
                "failed",
            ]
            .contains(&status)
            {
                return Err(Error::Configuration("Unsupported status"));
            }
        }
        let mut request = self
            .client
            .request(reqwest::Method::GET, "documents")?
            .query(&[("limit", limit.to_string())]);
        if let Some(status) = status {
            request = request.query(&[("status", status)]);
        }
        let body = self.client.send(request)?;
        body.get("documents")
            .and_then(Value::as_array)
            .cloned()
            .ok_or(Error::MalformedResponse)
    }

    pub fn create(&self, input: CreateDocument<'_>) -> Result<Value, Error> {
        if input.name.trim().is_empty()
            || input
                .fields
                .as_ref()
                .is_some_and(|fields| fields.len() > 400)
        {
            return Err(Error::Configuration(
                "Name is required and fields are limited to 400",
            ));
        }
        self.client.send(
            self.client
                .request(reqwest::Method::POST, "documents")?
                .json(&input),
        )
    }

    pub fn upload(&self, input: Upload<'_>) -> Result<Value, Error> {
        if !(1..=25).contains(&input.pages.len()) {
            return Err(Error::Configuration("Upload requires 1 to 25 page images"));
        }
        let mut form = multipart::Form::new();
        for path in input.pages {
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !["jpg", "jpeg", "png", "webp", "heic"].contains(&extension.as_str()) {
                return Err(Error::Configuration(
                    "Upload accepts JPEG, PNG, WebP or HEIC images",
                ));
            }
            if path.metadata()?.len() > 100 * 1024 * 1024 {
                return Err(Error::Configuration("Each page must be at most 100 MB"));
            }
            form = form.file("file", path)?;
        }
        if let Some(name) = input.name {
            form = form.text("name", name.to_owned());
        }
        if let Some(template_id) = input.template_id {
            form = form.text("template_id", template_id.to_owned());
        }
        if input.asynchronous {
            form = form.text("async", "true");
        }
        self.client.send(
            self.client
                .request(reqwest::Method::POST, "documents/upload")?
                .multipart(form),
        )
    }

    pub fn get(&self, id: &str) -> Result<Value, Error> {
        let url = self.client.document_url(id)?;
        self.client
            .send(self.client.http.get(url).bearer_auth(&self.client.api_key))
    }

    pub fn update(&self, id: &str, input: UpdateDocument) -> Result<Value, Error> {
        if input.status.is_none() && input.fields.is_none() {
            return Err(Error::Configuration("A change is required"));
        }
        if input
            .status
            .as_deref()
            .is_some_and(|status| !["review", "approved", "exported"].contains(&status))
        {
            return Err(Error::Configuration("Unsupported status"));
        }
        let url = self.client.document_url(id)?;
        self.client.send(
            self.client
                .http
                .patch(url)
                .bearer_auth(&self.client.api_key)
                .json(&input),
        )
    }
}

pub struct Workflows<'a> {
    client: &'a Client,
}

impl Workflows<'_> {
    pub fn dispatch(
        &self,
        event: &str,
        document_id: &str,
        payload: Option<Value>,
    ) -> Result<Value, Error> {
        if ![
            "document.processed",
            "document.needs_review",
            "document.approved",
            "document.exported",
        ]
        .contains(&event)
            || document_id.is_empty()
        {
            return Err(Error::Configuration(
                "A documented event and document ID are required",
            ));
        }
        let mut body = serde_json::json!({"event": event, "document_id": document_id});
        if let Some(payload) = payload {
            body["payload"] = payload;
        }
        self.client.send(
            self.client
                .request(reqwest::Method::POST, "workflows/dispatch")?
                .json(&body),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};
    use std::fs;

    #[test]
    fn constructs_document_and_workflow_requests() {
        let mut server = Server::new();
        let list = server
            .mock("GET", "/api/public/v1/documents")
            .match_header("authorization", "Bearer test-key")
            .match_query(Matcher::Exact("limit=10".into()))
            .with_body(r#"{"documents":[{"id":"example"}]}"#)
            .create();
        let create = server
            .mock("POST", "/api/public/v1/documents")
            .match_body(Matcher::PartialJsonString(r#"{"name":"Invoice"}"#.into()))
            .with_body(r#"{"document":{"id":"example"}}"#)
            .create();
        let get = server
            .mock("GET", "/api/public/v1/documents/doc-id")
            .with_body(r#"{"document":{"id":"doc-id"}}"#)
            .create();
        let update = server
            .mock("PATCH", "/api/public/v1/documents/doc-id")
            .match_body(Matcher::PartialJsonString(
                r#"{"status":"approved"}"#.into(),
            ))
            .with_body(r#"{"ok":true,"updated_fields":0}"#)
            .create();
        let dispatch = server
            .mock("POST", "/api/public/v1/workflows/dispatch")
            .match_body(Matcher::PartialJsonString(
                r#"{"event":"document.approved","document_id":"doc-id"}"#.into(),
            ))
            .with_body(r#"{"ok":true}"#)
            .create();
        let client = Client::with_base_url(
            "test-key",
            &format!("{}/api/public/v1", server.url()),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(client.documents().list(10, None).unwrap().len(), 1);
        client
            .documents()
            .create(CreateDocument {
                name: "Invoice",
                fields: None,
                category: None,
                vendor: None,
                amount: None,
                document_date: None,
            })
            .unwrap();
        client.documents().get("doc-id").unwrap();
        client
            .documents()
            .update(
                "doc-id",
                UpdateDocument {
                    status: Some("approved".into()),
                    fields: None,
                },
            )
            .unwrap();
        client
            .workflows()
            .dispatch("document.approved", "doc-id", None)
            .unwrap();
        list.assert();
        create.assert();
        get.assert();
        update.assert();
        dispatch.assert();
    }

    #[test]
    fn uploads_repeated_image_fields_and_rejects_pdf() {
        let mut server = Server::new();
        let upload = server
            .mock("POST", "/api/public/v1/documents/upload")
            .match_header("content-type", Matcher::Regex("multipart/form-data".into()))
            .match_body(Matcher::Regex("name=\\\"file\\\"".into()))
            .with_body(r#"{"document":{"id":"example"}}"#)
            .create();
        let client = Client::with_base_url(
            "test-key",
            &format!("{}/api/public/v1", server.url()),
            Duration::from_secs(5),
        )
        .unwrap();
        let image = std::env::temp_dir().join(format!("formalini-{}.jpg", std::process::id()));
        fs::write(&image, b"image fixture").unwrap();
        let result = client.documents().upload(Upload {
            pages: std::slice::from_ref(&image),
            name: Some("Example"),
            template_id: None,
            asynchronous: true,
        });
        fs::remove_file(&image).unwrap();
        result.unwrap();
        upload.assert();
        assert!(matches!(
            client.documents().upload(Upload {
                pages: &[PathBuf::from("file.pdf")],
                name: None,
                template_id: None,
                asynchronous: false
            }),
            Err(Error::Configuration(_))
        ));
    }

    #[test]
    fn maps_api_errors_and_validates_input() {
        let mut server = Server::new();
        let denied = server
            .mock("GET", "/api/public/v1/documents/doc-id")
            .with_status(403)
            .with_body(r#"{"error":"Insufficient scope."}"#)
            .create();
        let client = Client::with_base_url(
            "test-key",
            &format!("{}/api/public/v1", server.url()),
            Duration::from_secs(5),
        )
        .unwrap();
        match client.documents().get("doc-id") {
            Err(Error::Api {
                status, message, ..
            }) => {
                assert_eq!(status, StatusCode::FORBIDDEN);
                assert_eq!(message, "Insufficient scope.");
            }
            _ => panic!("expected API error"),
        }
        denied.assert();
        assert!(matches!(
            client.documents().list(0, None),
            Err(Error::Configuration(_))
        ));
        assert!(Client::new("").is_err());
    }

    #[test]
    fn maps_usage_limit_error() {
        let mut server = Server::new();
        let limited = server
            .mock("GET", "/api/public/v1/documents/doc-id")
            .with_status(402)
            .with_body(r#"{"error":"No credits remaining"}"#)
            .create();
        let client = Client::with_base_url(
            "test-key",
            &format!("{}/api/public/v1", server.url()),
            Duration::from_secs(5),
        )
        .unwrap();
        match client.documents().get("doc-id") {
            Err(Error::UsageLimit { message }) => assert_eq!(message, "No credits remaining"),
            _ => panic!("expected usage-limit error"),
        }
        limited.assert();
    }
}
