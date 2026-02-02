use anyhow::Result;
use chrono::FixedOffset;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use lorm::ToLOrm;
use lorm::predicates::{Function, Having, Where};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Executor, FromRow, Sqlite, SqlitePool};
use std::ops::Add;
use std::time::Duration;
use tokio::fs;
use tokio::time::{Instant, sleep_until};
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
    #[lorm(skip)]
    #[sqlx(skip)]
    pub tmp: i64,

    #[lorm(created_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

/// Alternative user specifically for testing id with another type.
#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct AltUser {
    #[lorm(pk)]
    #[lorm(readonly)]
    pub id: i32,

    #[lorm(by)]
    pub email: String,

    #[lorm(by)]
    pub count: Option<i32>,

    #[allow(unused)]
    #[lorm(created_at)]
    #[lorm(readonly)]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

pub async fn get_conn(pool: SqlitePool) -> SqlitePool {
    pool
}

pub async fn get_pool() -> Result<SqlitePool> {
    let database_name = Uuid::new_v4().to_string();
    let mut db_path = std::env::temp_dir();
    db_path = db_path.join(format!("{}.db", database_name));
    let database_url = format!("sqlite://{}", db_path.display());

    if Sqlite::database_exists(&database_url).await? == true {
        Sqlite::drop_database(&database_url).await?;
    }
    Sqlite::create_database(&database_url).await?;
    let pool = SqlitePool::connect(&database_url).await?;
    let migration_path = fs::canonicalize("tests/resources/migrations").await?;
    let mut dir = fs::read_dir(migration_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let bytes = fs::read(entry.path()).await?;
        let content = String::from_utf8(bytes)?;
        pool.execute(content.as_str()).await?;
    }
    Ok(pool)
}

#[tokio::test]
async fn test_user_does_not_exists() {
    let pool = get_pool().await.expect("Failed to create pool");

    let email = SafeEmail().fake::<String>();
    let res = User::by_email(&pool, email).await;
    assert_eq!(res.is_err(), true);

    let id = Uuid::new_v4();
    let res = User::by_id(&pool, id).await;
    assert_eq!(res.is_err(), true);
}

#[tokio::test]
async fn test_user_is_created() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = User::default();
    let email = SafeEmail().fake::<String>();
    u.email = email.clone();
    let u = u.save(&pool).await.unwrap();

    let res = User::by_id(&pool, u.id).await;
    assert_eq!(res.is_err(), false);

    let u = res.unwrap();
    assert_eq!(u.created_at.to_rfc2822() == u.updated_at.to_rfc2822(), true);

    let res = User::by_email(&pool, email).await;
    assert_eq!(res.is_err(), false);
}

#[tokio::test]
async fn test_user_is_updated() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = User::default();
    u.email = SafeEmail().fake::<String>();
    let mut u = u.save(&pool).await.unwrap();

    // Needed for the created_at != updated_at assertion.
    let _ = sleep_until(Instant::now().add(Duration::from_secs(1))).await;

    let email = SafeEmail().fake::<String>();
    u.email = email.clone();
    let u = u.save(&pool).await.unwrap();
    let res = User::by_id(&pool, u.id).await;
    assert_eq!(res.is_err(), false);
    let u = res.unwrap();
    assert_eq!(u.email, email);
    assert_eq!(u.created_at.to_rfc3339() != u.updated_at.to_rfc3339(), true);
}

#[tokio::test]
async fn test_user_is_deleted() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = User::default();
    u.email = SafeEmail().fake::<String>();
    let u = u.save(&pool).await.unwrap();

    let res = User::by_id(&pool, u.id).await;
    assert_eq!(res.is_err(), false);

    u.delete(&pool).await.unwrap();
    let res = User::by_id(&pool, u.id).await;
    assert_eq!(res.is_err(), true);
}

#[tokio::test]
async fn test_user_are_listed() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_users(&pool, 10, None).await;

    let res = User::select().limit(2).build(&pool).await.unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
}

#[tokio::test]
async fn test_with_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_alt_users(&pool, 10).await;
    let _ = create_alt_users(&pool, 11).await;
    let res = AltUser::with_count(&pool, 5).await.unwrap();
    assert_eq!(res.len(), 2);
    let res = AltUser::with_count(&pool, 10).await.unwrap();
    assert_eq!(res.len(), 1);
    let res = AltUser::with_count(&pool, 11).await.unwrap();
    assert_eq!(res.len(), 0);
}

#[tokio::test]
async fn test_offset_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let users = create_users(&pool, 10, None).await;

    let res = User::select()
        .order_by_email()
        .desc()
        .limit(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, users.get(8).unwrap().email);

    let res = User::select()
        .order_by_email()
        .desc()
        .limit(2)
        .offset(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.last().unwrap();
    assert_eq!(u.email, users.get(6).unwrap().email);
}

#[tokio::test]
async fn test_group_by_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_users(&pool, 11, None).await;
    let other_users = create_users(&pool, 11, None).await;

    let res = User::select()
        .group_by_email()
        .order_by_created_at()
        .desc()
        .limit(2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.is_empty(), false);
    assert_eq!(res.len(), 2);
    let u = res.first().unwrap();
    assert_eq!(u.email, other_users.get(10).unwrap().email);
}

#[tokio::test]
async fn test_automatic_pk_and_ts_insertion_update_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = AltUser::default();
    u.email = SafeEmail().fake::<String>();
    let u = u.save(&pool).await.unwrap();
    let res = AltUser::by_id(&pool, u.id).await;
    assert_eq!(res.is_err(), false);
}

#[tokio::test]
async fn test_where_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let users = create_alt_users(&pool, 10).await;
    let u = users.get(2).unwrap();

    let res = AltUser::select()
        .where_id(Where::Eq, u.id)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 1);
}

#[tokio::test]
async fn test_between_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_alt_users(&pool, 10).await;

    let res = AltUser::select()
        .where_between_count(2, 4)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 3);
}

#[tokio::test]
async fn test_like_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_alt_users(&pool, 11).await;

    let res = AltUser::select()
        .where_email(Where::Like, "1%")
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 2);

    let res = AltUser::select()
        .where_email(Where::Like, "%")
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 11);
}

#[tokio::test]
async fn test_having_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let _ = create_alt_users(&pool, 10).await;
    let res = AltUser::select()
        .group_by_count()
        .having_all_count(Having::Eq, 2)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = AltUser::select()
        .group_by_count()
        .having_count(Having::Eq, Function::Max, 1)
        .build(&pool)
        .await
        .unwrap();
    assert_eq!(res.len(), 1);
}

async fn create_users<'e, E: sqlx::SqliteExecutor<'e> + Copy>(
    conn: E,
    count: i32,
    prefix: Option<&'static str>,
) -> Vec<User> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = User::default();
        u.email = match prefix {
            None => format!("{i}-{email}").to_string(),
            Some(v) => format!("{v}-{email}").to_string(),
        };
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

async fn create_alt_users<'e, E: sqlx::SqliteExecutor<'e> + Copy>(
    conn: E,
    count: i32,
) -> Vec<AltUser> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = AltUser::default();
        u.email = format!("{i}-{email}").to_string();
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}
