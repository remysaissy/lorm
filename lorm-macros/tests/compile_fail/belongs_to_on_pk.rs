use lorm::ToLOrm;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
pub struct BadStruct {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    #[lorm(is_set = "Uuid::is_nil")]
    #[lorm(belongs_to = User)]
    pub id: Uuid,
    pub name: String,
}

fn main() {}
