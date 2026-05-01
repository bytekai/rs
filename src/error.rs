use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum AppError {
    NotFound,
    Conflict { on: Option<String> },
    Db(sqlx::Error),
    Validation(validator::ValidationErrors),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        if matches!(e, sqlx::Error::RowNotFound) {
            return Self::NotFound;
        }
        if let Some(d) = e.as_database_error()
            && matches!(d.kind(), sqlx::error::ErrorKind::UniqueViolation)
        {
            return Self::Conflict {
                on: parse_unique_column(d.message()),
            };
        }
        Self::Db(e)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        Self::Validation(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Validation(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
            }
            Self::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
            Self::Conflict { on: Some(c) } => {
                (StatusCode::CONFLICT, format!("conflict on `{}`", c)).into_response()
            }
            Self::Conflict { .. } => (StatusCode::CONFLICT, "conflict").into_response(),
            Self::Db(e) => {
                tracing::error!(?e, "db error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal").into_response()
            }
        }
    }
}

// SQLite unique constraint message: "UNIQUE constraint failed: users.email"
fn parse_unique_column(msg: &str) -> Option<String> {
    msg.strip_prefix("UNIQUE constraint failed: ")
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.split('.').nth(1))
        .map(|s| s.trim().to_string())
}
