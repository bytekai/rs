use crate::error::AppError;
use crate::task::model::{Task, TaskCreate, TaskUpdate};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

pub async fn create(
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

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
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

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, AppError> {
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

pub async fn update(
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

pub async fn delete(
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
