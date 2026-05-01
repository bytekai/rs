mod handler;
mod model;

use axum::Router;
use axum::routing as r;
use sqlx::SqlitePool;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/tasks", r::post(handler::create).get(handler::list))
        .route(
            "/tasks/{id}",
            r::get(handler::get)
                .patch(handler::update)
                .delete(handler::delete),
        )
}
