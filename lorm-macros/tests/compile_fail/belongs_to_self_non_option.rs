use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
pub struct Category {
    #[lorm(belongs_to = Self)]
    pub parent_id: Uuid,
    pub name: String,
}

fn main() {}
