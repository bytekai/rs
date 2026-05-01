use chrono::{DateTime, Utc};
use entity::model;
use uuid::Uuid;

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
