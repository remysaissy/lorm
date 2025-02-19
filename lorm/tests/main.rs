use chrono::FixedOffset;
use lorm::predicates::OrderBy;
use lorm::ToLOrm;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Executor, FromRow, Sqlite, SqlitePool};
use std::thread::sleep;
use std::time::Duration;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct User {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    // Default is used but a boolean check function can also be used.
    #[lorm(is_set = "is_nil()")]
    pub id: Uuid,

    #[lorm(by)]
    pub email: String,

    #[allow(unused)]
    #[lorm(readonly)]
    pub count: Option<i32>,

    #[allow(unused)]
    #[lorm(transient)]
    #[sqlx(skip)]
    pub tmp: i64,

    #[lorm(created_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct Post {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    pub id: Uuid,

    pub content: String,

    #[lorm(fk = "User")]
    pub user_id: Uuid,
}

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct AltUser {
    #[lorm(pk)]
    #[lorm(readonly)]
    pub id: i32,

    pub email: String,

    #[lorm(by)]
    pub count: i32,

    #[allow(unused)]
    #[lorm(created_at)]
    #[lorm(readonly)]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

pub async fn get_pool() -> SqlitePool {
    let database_name = Uuid::new_v4().to_string();
    let mut db_path = std::env::temp_dir();
    db_path = db_path.join(format!("{}.db", database_name));
    let database_url = format!("sqlite://{}", db_path.display());

    if Sqlite::database_exists(&database_url).await.unwrap() == true {
        Sqlite::drop_database(&database_url).await.unwrap();
    }
    Sqlite::create_database(&database_url).await.unwrap();
    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let migration_path = fs::canonicalize("tests/resources/migrations")
        .await
        .unwrap();
    let mut dir = fs::read_dir(migration_path).await.unwrap();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let bytes = fs::read(entry.path()).await.unwrap();
        let content = String::from_utf8(bytes).unwrap();
        pool.execute(content.as_str()).await.unwrap();
    }
    pool
}

#[tokio::test]
async fn test_user_does_not_exists() {
    let pool = get_pool().await;
    let res = User::by_email(&pool, "alice.dupont@domain.com".to_string())
        .await
        .unwrap();
    assert_eq!(res.is_none(), true);

    let id = Uuid::new_v4();
    let res = User::by_id(&pool, id).await.unwrap();
    assert_eq!(res.is_none(), true);
}

#[tokio::test]
async fn test_user_is_created() {
    let pool = get_pool().await;
    let mut u = User::default();
    u.email = "alice.dupont@domain.com".to_string();
    let u = u.save(&pool).await.unwrap();

    let res = User::by_id(&pool, u.id.clone()).await.unwrap();
    assert_eq!(res.is_none(), false);

    let u = res.unwrap();
    assert_eq!(u.created_at.to_rfc2822() == u.updated_at.to_rfc2822(), true);
}

#[tokio::test]
async fn test_post_is_created() {
    let pool = get_pool().await;
    let mut u = User::default();
    u.email = "alice.dupont@domain.com".to_string();
    let u = u.save(&pool).await.unwrap();

    let mut p = Post::default();
    p.content = "hello world".to_string();
    p.user_id = u.id;
    let p = p.save(&pool).await.unwrap();

    let u2 = p.get_user(&pool).await;
    assert_eq!(u2.is_err(), false);
    let u2 = u2.unwrap();
    assert_eq!(u2.is_some(), true);
    let u2 = u2.unwrap();
    assert_eq!(u2.id, u.id);
}

#[tokio::test]
async fn test_user_is_updated() {
    let pool = get_pool().await;
    let mut u = User::default();
    u.email = "alice.dupont@domain.com".to_string();
    let mut u = u.save(&pool).await.unwrap();

    // Needed for the created_at != updated_at assertion.
    sleep(Duration::from_secs(1));

    u.email = "alice.dupont@new-domain.com".to_string();
    let u = u.save(&pool).await.unwrap();
    let res = User::by_id(&pool, u.id.clone()).await.unwrap();
    assert_eq!(res.is_none(), false);
    let u = res.unwrap();
    assert_eq!(u.email, "alice.dupont@new-domain.com".to_string());
    assert_eq!(u.created_at.to_rfc3339() != u.updated_at.to_rfc3339(), true);
}

#[tokio::test]
async fn test_user_is_deleted() {
    let pool = get_pool().await;
    let mut u = User::default();
    u.email = "alice.dupont@domain.com".to_string();
    let u = u.save(&pool).await.unwrap();

    let res = User::by_id(&pool, u.id.clone()).await.unwrap();
    assert_eq!(res.is_none(), false);

    u.delete(&pool).await.unwrap();
    let res = User::by_id(&pool, u.id.clone()).await.unwrap();
    assert_eq!(res.is_none(), true);
}

#[tokio::test]
async fn test_user_are_listed() {
    let pool = get_pool().await;
    for i in 0..10 {
        let mut u = User::default();
        u.email = format!("alice.dupont@domain-{i}.com").to_string();
        let _ = u.save(&pool).await.unwrap();
    }

    let res = User::select().limit(2).build(&pool).await.unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
}

#[tokio::test]
async fn test_offset_is_working() {
    let pool = get_pool().await;
    for i in 0..10 {
        let mut u = User::default();
        u.email = format!("alice.dupont@domain-{i}.com").to_string();
        let _ = u.save(&pool).await.unwrap();
    }

    let res = User::select()
        .order_by_email(OrderBy::Desc)
        .limit(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, "alice.dupont@domain-8.com");

    let res = User::select()
        .order_by_email(OrderBy::Desc)
        .limit(2)
        .offset(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, "alice.dupont@domain-6.com");
}

#[tokio::test]
async fn test_group_by_is_working() {
    let pool = get_pool().await;
    for i in 0..10 {
        let mut u = User::default();
        u.email = format!("alice.dupont@domain-{i}.com").to_string();
        let _ = u.save(&pool).await.unwrap();
    }

    for i in 0..10 {
        let mut u = User::default();
        u.email = format!("jean.dupont@domain-{i}.com").to_string();
        let _ = u.save(&pool).await.unwrap();
    }

    let res = User::select()
        .group_by_email()
        .group_by_id()
        .order_by_email(OrderBy::Desc)
        .limit(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, "jean.dupont@domain-8.com");

    let res = User::select()
        .group_by_email()
        .group_by_id()
        .order_by_email(OrderBy::Desc)
        .limit(2)
        .offset(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, "jean.dupont@domain-6.com");

    let res = User::select()
        .group_by_email()
        .group_by_id()
        .order_by_email(OrderBy::Asc)
        .limit(2)
        .offset(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, "alice.dupont@domain-3.com");
}

#[tokio::test]
async fn test_automatic_pk_and_ts_insertion_update_is_working() {
    let pool = get_pool().await;
    let mut u = AltUser::default();
    u.email = "alice.dupont@domain.com".to_string();
    let u = u.save(&pool).await.unwrap();
    let res = AltUser::by_id(&pool, u.id.clone()).await.unwrap();
    assert_eq!(res.is_none(), false);
}

#[tokio::test]
async fn test_where_is_working() {
    let pool = get_pool().await;
    let mut u = AltUser::default();
    u.email = "alice.dupont@domain.com".to_string();
    let u = u.save(&pool).await.unwrap();

    let res = AltUser::select()
        .where_id_equals(u.id)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 1);
}

#[tokio::test]
async fn test_between_is_working() {
    let pool = get_pool().await;
    for i in 0..10 {
        let mut u = AltUser::default();
        u.email = format!("jean.dupont@domain-{i}.com").to_string();
        u.count = i;
        let _ = u.save(&pool).await.unwrap();
    }

    let res = AltUser::select()
        .where_count_is_between(2, 4)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);
}
