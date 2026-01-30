use anyhow::Result;
use chrono::FixedOffset;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use lorm::ToLOrm;
use lorm::predicates::Where;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Executor, FromRow, Sqlite, SqliteConnection, SqlitePool};
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
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let email = SafeEmail().fake::<String>();
        let res = User::by_email(&mut *conn, email).await;
        assert_eq!(res.is_err(), true);

        let id = Uuid::new_v4();
        let res = User::by_id(&mut *conn, id).await;
        assert_eq!(res.is_err(), true);
    }
}

#[tokio::test]
async fn test_user_is_created() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let mut u = User::default();
        let email = SafeEmail().fake::<String>();
        u.email = email.clone();
        let u = u.save(&mut *conn).await.unwrap();

        let res = User::by_id(&mut *conn, u.id).await;
        assert_eq!(res.is_err(), false);

        let u = res.unwrap();
        assert_eq!(u.created_at.to_rfc2822() == u.updated_at.to_rfc2822(), true);

        let res = User::by_email(&mut *conn, email).await;
        assert_eq!(res.is_err(), false);
    }
}

#[tokio::test]
async fn test_user_is_updated() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let mut u = User::default();
        u.email = SafeEmail().fake::<String>();
        let mut u = u.save(&mut *conn).await.unwrap();

        // Needed for the created_at != updated_at assertion.
        let _ = sleep(Duration::from_secs(1));

        let email = SafeEmail().fake::<String>();
        u.email = email.clone();
        let u = u.save(&mut *conn).await.unwrap();
        let res = User::by_id(&mut *conn, u.id).await;
        assert_eq!(res.is_err(), false);
        let u = res.unwrap();
        assert_eq!(u.email, email);
        assert_eq!(u.created_at.to_rfc3339() != u.updated_at.to_rfc3339(), true);
    }
}

#[tokio::test]
async fn test_user_is_deleted() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let mut u = User::default();
        u.email = SafeEmail().fake::<String>();
        let u = u.save(&mut *conn).await.unwrap();

        let res = User::by_id(&mut *conn, u.id).await;
        assert_eq!(res.is_err(), false);

        u.delete(&mut *conn).await.unwrap();
        let res = User::by_id(&mut *conn, u.id).await;
        assert_eq!(res.is_err(), true);
    }
}

#[tokio::test]
async fn test_user_are_listed() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        for _ in 0..10 {
            let mut u = User::default();
            u.email = SafeEmail().fake::<String>();
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let res = User::select().limit(2).build(&mut *conn).await.unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
    }
}

#[tokio::test]
async fn test_with_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        for _ in 0..10 {
            let mut u = AltUser::default();
            u.email = SafeEmail().fake::<String>();
            u.count = Some(42);
            let _ = u.save(&mut *conn).await.unwrap();
        }
        let res = AltUser::with_count(&mut *conn, 42).await.unwrap();
        assert_eq!(res.len(), 10);
    }
}

#[tokio::test]
async fn test_offset_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let email = SafeEmail().fake::<String>();
        for i in 0..10 {
            let mut u = User::default();
            u.email = format!("{i}-{email}").to_string();
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let res = User::select()
            .order_by_email()
            .desc()
            .limit(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("8-{email}"));

        let res = User::select()
            .order_by_email()
            .desc()
            .limit(2)
            .offset(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("6-{email}"));
    }
}

#[tokio::test]
async fn test_group_by_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let email = SafeEmail().fake::<String>();
        for i in 0..10 {
            let mut u = User::default();
            u.email = format!("{i}-{email}").to_string();
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let other_email = format!("ZZZ-{email}");
        for i in 0..10 {
            let mut u = User::default();
            u.email = format!("{i}-{other_email}").to_string();
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let res = User::select()
            .group_by_email()
            .group_by_id()
            .order_by_email()
            .desc()
            .limit(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("9-{other_email}"));

        let res = User::select()
            .group_by_email()
            .group_by_id()
            .order_by_email()
            .desc()
            .limit(2)
            .offset(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("8-{other_email}"));

        let res = User::select()
            .group_by_email()
            .group_by_id()
            .order_by_email()
            .asc()
            .limit(2)
            .offset(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("1-{email}"));

        let res = User::select()
            .group_by_email()
            .group_by_id()
            .order_by_created_at()
            .asc()
            .limit(2)
            .offset(2)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.is_empty(), false);
        assert_eq!(res.len(), 2);
        let u = res.last().unwrap();
        assert_eq!(u.email, format!("3-{email}"));
    }
}

#[tokio::test]
async fn test_automatic_pk_and_ts_insertion_update_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let mut u = AltUser::default();
        u.email = SafeEmail().fake::<String>();
        let u = u.save(&mut *conn).await.unwrap();
        let res = AltUser::by_id(&mut *conn, u.id).await;
        assert_eq!(res.is_err(), false);
    }
}

#[tokio::test]
async fn test_where_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let mut u = AltUser::default();
        u.email = SafeEmail().fake::<String>();
        let u = u.save(&mut *conn).await.unwrap();

        let res = AltUser::select()
            .where_id(Where::Eq, u.id)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
    }
}

#[tokio::test]
async fn test_between_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let email = SafeEmail().fake::<String>();
        for i in 0..10 {
            let mut u = AltUser::default();
            u.email = format!("{i}-{email}").to_string();
            u.count = Some(i);
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let res = AltUser::select()
            .where_between_count(2, 4)
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.len(), 3);
    }
}

#[tokio::test]
async fn test_like_is_working() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let email = SafeEmail().fake::<String>();
        for i in 0..11 {
            let mut u = AltUser::default();
            u.email = format!("{i}-{email}").to_string();
            u.count = Some(i);
            let _ = u.save(&mut *conn).await.unwrap();
        }

        let res = AltUser::select()
            .where_email(Where::Like, "1%")
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.len(), 2);

        let res = AltUser::select()
            .where_email(Where::Like, "%")
            .build(&mut *conn)
            .await
            .unwrap();
        assert_eq!(res.len(), 11);
    }
}
