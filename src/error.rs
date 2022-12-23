// Based on https://fasterthanli.me/series/updating-fasterthanli-me-for-2022/part-2.

use axum::{
    http::{self, StatusCode},
    response::{IntoResponse, Response},
};
use color_eyre::Report;
use tracing::error;

pub type HttpResult = Result<Response, HttpError>;

pub trait IntoHttp {
    fn into_http(self) -> HttpResult;
}

impl<T: IntoResponse> IntoHttp for T {
    fn into_http(self) -> HttpResult {
        Ok(self.into_response())
    }
}

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

            let backtrace: String = if let Some(bt) = maybe_bt {
                format!("{:?}", bt)
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
impl_from!(serde_json::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            // TODO: display 404 file.
            HttpError::NotFound => (StatusCode::NOT_FOUND, "").into_response(),
            HttpError::Internal { err } => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
        }
    }
}

// TODO: check environment variables.
fn is_production() -> bool {
    false
}
