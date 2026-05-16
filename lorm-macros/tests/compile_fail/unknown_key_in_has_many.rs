use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
#[lorm(has_many(Post, unknown = "x"))]
pub struct User {
    #[lorm(pk)]
    pub id: Uuid,
}

fn main() {}
