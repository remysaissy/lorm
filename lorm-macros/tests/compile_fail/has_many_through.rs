use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

// `through` is not supported for has_many/has_one (no many-to-many).
#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
#[lorm(has_many(Post, through = "Tag"))]
pub struct User {
    #[lorm(pk)]
    pub id: Uuid,
}

fn main() {}
