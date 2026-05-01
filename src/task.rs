use crate::error::AppError;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing as r;
use chrono::{DateTime, Utc};
use entity::model;
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

#[model]
pub struct Task {
    #[model(skip)]
    pub id: Uuid,

    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 2000))]
    pub description: String,

    pub done: bool,

    #[model(skip)]
    pub created_at: DateTime<Utc>,
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/tasks", r::post(create).get(list))
        .route("/tasks/{id}", r::get(get).patch(update).delete(delete))
}

async fn create(
    State(pool): State<SqlitePool>,
    Json(input): Json<TaskCreate>,
) -> Result<Json<Task>, AppError> {
    input.validate()?;
    let id = Uuid::now_v7();
    let created_at = Utc::now();
    sqlx::query!(
        "INSERT INTO tasks (id, title, description, done, created_at) VALUES (?, ?, ?, ?, ?)",
        id,
        input.title,
        input.description,
        input.done,
        created_at,
    )
    .execute(&pool)
    .await?;
    Ok(Json(Task {
        id,
        title: input.title,
        description: input.description,
        done: input.done,
        created_at,
    }))
}

async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
    let xs = sqlx::query_as!(
        Task,
        r#"SELECT
            id          as "id!: Uuid",
            title       as "title!: String",
            description as "description!: String",
            done        as "done!: bool",
            created_at  as "created_at!: DateTime<Utc>"
        FROM tasks"#,
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(xs))
}

async fn get(State(pool): State<SqlitePool>, Path(id): Path<Uuid>) -> Result<Json<Task>, AppError> {
    let t = sqlx::query_as!(
        Task,
        r#"SELECT
            id          as "id!: Uuid",
            title       as "title!: String",
            description as "description!: String",
            done        as "done!: bool",
            created_at  as "created_at!: DateTime<Utc>"
        FROM tasks WHERE id = ?"#,
        id,
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(t))
}

async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(p): Json<TaskUpdate>,
) -> Result<StatusCode, AppError> {
    p.validate()?;
    let rows = sqlx::query!(
        r#"UPDATE tasks SET
            title       = COALESCE(?, title),
            description = COALESCE(?, description),
            done        = COALESCE(?, done)
        WHERE id = ?"#,
        p.title,
        p.description,
        p.done,
        id,
    )
    .execute(&pool)
    .await?
    .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let rows = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
        .execute(&pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
