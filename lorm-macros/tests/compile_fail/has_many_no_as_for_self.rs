use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
#[lorm(has_many = Self)]
pub struct Node {
    #[lorm(pk)]
    pub id: Uuid,
    pub name: String,
}

fn main() {}
