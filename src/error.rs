// Based on https://fasterthanli.me/series/updating-fasterthanli-me-for-2022/part-2.

use std::{
    io::{BufReader, Read},
    sync::OnceLock,
};

use axum::{
    http::{self, header, StatusCode},
    response::{IntoResponse, Response},
};
use color_eyre::Report;
use tracing::error;

#[derive(Debug)]
pub enum HttpError {
    NotFound,
    Internal { err: String },
}

impl HttpError {
    fn from_report(err: Report) -> Self {
        error!("HTTP handler error: {}", err.root_cause());

        let maybe_bt = err
            .context()
            .downcast_ref::<color_eyre::Handler>()
            .and_then(|h| h.backtrace());
        if let Some(bt) = maybe_bt {
            error!("Backtrace: {:?}", bt);
        } else {
            error!("No Backtrace");
        }

        let trace_content = if is_production() {
            "".into()
        } else {
            let mut err_string = String::new();
            for (i, e) in err.chain().enumerate() {
                use std::fmt::Write;
                let _ = writeln!(&mut err_string, "{}. {}", i + 1, e);
            }

            let err_string = html_escape::encode_safe(&err_string);

            let backtrace: String = if let Some(bt) = maybe_bt {
                let backtrace = format!("{:?}", bt);
                html_escape::encode_safe(&backtrace).into()
            } else {
                "".into()
            };

            format!(
                r#"{err_string}
{backtrace}"#
            )
        };

        let body = trace_content;

        HttpError::Internal { err: body }
    }
}

macro_rules! impl_from {
    ($from:ty) => {
        impl From<$from> for HttpError {
            fn from(err: $from) -> Self {
                Self::from_report(err.into())
            }
        }
    };
}

impl_from!(std::io::Error);
impl_from!(color_eyre::Report);
impl_from!(http::Error);
impl_from!(http::uri::InvalidUri);
impl_from!(http::uri::InvalidUriParts);
impl_from!(hyper::Error);
impl_from!(hyper_util::client::legacy::Error);
impl_from!(serde_json::Error);

const CONTENT_TYPE_HTML: &str = "text/html";

/// Reads the contents of the 404 and 500 error pages into memory,
/// panicking if the files are not found.
pub fn read_error_file_contents() {
    error_404_contents();
    error_500_contents();
}

fn error_404_contents() -> &'static [u8] {
    static ERROR_404: OnceLock<Vec<u8>> = OnceLock::new();
    &ERROR_404.get_or_init(|| {
        let file = std::fs::File::open("dist/404.html").expect("could not open 404 file");
        let mut reader = BufReader::new(file);
        let mut contents = vec![];
        reader.read_to_end(&mut contents).unwrap();
        contents
    })
}

fn error_500_contents() -> &'static str {
    static ERROR_500: OnceLock<String> = OnceLock::new();
    &ERROR_500.get_or_init(|| {
        let file = std::fs::File::open("dist/404.html").expect("could not open 500 file");
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();
        contents
    })
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::NotFound => (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, CONTENT_TYPE_HTML)],
                error_404_contents(),
            )
                .into_response(),
            HttpError::Internal { err } => {
                let err = format!("<pre>{err}</pre>");
                let contents = error_500_contents().replace("<!-- ERROR -->", &err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, CONTENT_TYPE_HTML)],
                    contents,
                )
                    .into_response()
            }
        }
    }
}

fn is_production() -> bool {
    std::env::var("FLY_APP_NAME")
        .map(|app_name| app_name == "ofcrse")
        .unwrap_or(false)
}
