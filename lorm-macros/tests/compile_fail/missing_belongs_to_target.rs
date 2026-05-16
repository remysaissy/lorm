use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
pub struct Link {
    #[lorm(belongs_to)]
    pub other_id: Uuid,
}

fn main() {}
