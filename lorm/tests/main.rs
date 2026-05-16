use anyhow::Result;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use lorm::predicates::{Function, Having, Where};
use sqlx::Executor;
use std::ops::Add;
use std::time::Duration;
use tokio::fs;
use tokio::time::{Instant, sleep_until};
use uuid::Uuid;

#[cfg(feature = "sqlite")]
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};

#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;

#[cfg(feature = "sqlite")]
type Pool = SqlitePool;
#[cfg(feature = "postgres")]
type Pool = PgPool;
#[cfg(feature = "mysql")]
type Pool = MySqlPool;

#[cfg(any(feature = "sqlite", feature = "postgres"))]
mod models {
    use chrono::FixedOffset;
    use lorm::ToLOrm;
    use sqlx::FromRow;
    use uuid::Uuid;

    #[derive(Debug, Default, Clone, sqlx::FromRow)]
    pub struct Address {
        pub street: String,
        #[sqlx(rename = "zip_code")]
        pub zip: String,
    }

    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    #[lorm(has_many = Post)]
    #[lorm(has_one = Profile)]
    pub struct User {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        // Default is used but a boolean check function can also be used.
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,

        #[lorm(by)]
        pub email: String,

        #[allow(unused)]
        #[lorm(readonly)]
        pub count: Option<i32>,

        #[allow(unused)]
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
    pub struct AltUser {
        #[lorm(pk)]
        #[lorm(readonly)]
        pub id: i32,

        #[lorm(by)]
        #[sqlx(rename = "e_mail")]
        pub email: String,

        #[lorm(by)]
        pub count: Option<i32>,

        #[allow(unused)]
        #[lorm(created_at)]
        #[lorm(readonly)]
        pub created_at: chrono::DateTime<FixedOffset>,

        #[allow(unused)]
        #[lorm(updated_at)]
        #[lorm(new = "chrono::Utc::now().fixed_offset()")]
        pub updated_at: chrono::DateTime<FixedOffset>,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, ToLOrm)]
    pub struct Profile {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub user_id: Uuid,
        #[sqlx(json)]
        pub preferences: serde_json::Value,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    pub struct Customer {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub email: String,
        #[sqlx(flatten)]
        #[lorm(flattened(street: String, zip: String = "zip_code"))]
        pub address: Address,
    }

    #[derive(Debug, Default, Clone, lorm::ToLOrm)]
    pub struct OptCustomer {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub email: String,
        #[sqlx(flatten)]
        #[lorm(flattened(street: String, zip: String = "zip_code"))]
        pub address: Option<Address>,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(pk_type = "manual")]
    pub struct UserRole {
        #[lorm(pk)]
        pub user_id: String,
        #[lorm(pk)]
        pub role_id: String,
        pub assigned_at: String,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(
        rename = "user_roles_named",
        pk_type = "manual",
        pk_selector = "named_by_user_role"
    )]
    pub struct UserRoleNamed {
        #[lorm(pk)]
        pub user_id: Uuid,
        #[lorm(pk)]
        pub role_name: String,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(pk_type = "manual")]
    pub struct Tag {
        #[lorm(pk)]
        pub name: String,
    }

    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    #[lorm(has_many(Self, fk = "parent_id", as = "children"))]
    pub struct Category {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        pub name: String,
        #[lorm(belongs_to = Self)]
        pub parent_id: Option<Uuid>,
    }

    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct Post {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        pub title: String,
        pub published: bool,
        #[lorm(belongs_to = User)]
        pub user_id: Uuid,
    }

    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct Draft {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        pub title: String,
        #[lorm(belongs_to = User)]
        pub user_id: Option<Uuid>,
    }

    #[cfg(feature = "sqlite")]
    impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for OptCustomer {
        fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
            use sqlx::Row;

            let id: Uuid = row.try_get("id")?;
            let email: String = row.try_get("email")?;
            let street: Option<String> = row.try_get("street")?;
            let zip: Option<String> = row.try_get("zip_code")?;
            let address = match (street, zip) {
                (None, None) => None,
                (Some(street), Some(zip)) => Some(Address { street, zip }),
                _ => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "partial NULL in flattened Option<Address>",
                    );
                    return Err(sqlx::Error::Decode(Box::new(err)));
                }
            };

            Ok(Self { id, email, address })
        }
    }

    #[cfg(feature = "postgres")]
    impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for OptCustomer {
        fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
            use sqlx::Row;

            let id: Uuid = row.try_get("id")?;
            let email: String = row.try_get("email")?;
            let street: Option<String> = row.try_get("street")?;
            let zip: Option<String> = row.try_get("zip_code")?;
            let address = match (street, zip) {
                (None, None) => None,
                (Some(street), Some(zip)) => Some(Address { street, zip }),
                _ => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "partial NULL in flattened Option<Address>",
                    );
                    return Err(sqlx::Error::Decode(Box::new(err)));
                }
            };

            Ok(Self { id, email, address })
        }
    }
}

#[cfg(feature = "mysql")]
mod models {
    use chrono::Utc;
    use lorm::ToLOrm;
    use sqlx::FromRow;
    use uuid::Uuid;

    #[derive(Debug, Default, Clone, sqlx::FromRow)]
    pub struct Address {
        pub street: String,
        #[sqlx(rename = "zip_code")]
        pub zip: String,
    }

    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct User {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        // Default is used but a boolean check function can also be used.
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,

        #[lorm(by)]
        pub email: String,

        #[allow(unused)]
        #[lorm(readonly)]
        pub count: Option<i32>,

        #[allow(unused)]
        #[sqlx(skip)]
        pub tmp: i64,

        #[lorm(created_at)]
        #[lorm(new = "chrono::Utc::now()")]
        pub created_at: chrono::DateTime<Utc>,

        #[lorm(updated_at)]
        #[lorm(new = "chrono::Utc::now()")]
        pub updated_at: chrono::DateTime<Utc>,
    }

    /// Alternative user specifically for testing id with another type.
    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct AltUser {
        #[lorm(pk)]
        #[lorm(readonly)]
        pub id: i32,

        #[lorm(by)]
        #[sqlx(rename = "e_mail")]
        pub email: String,

        #[lorm(by)]
        pub count: Option<i32>,

        #[allow(unused)]
        #[lorm(created_at)]
        #[lorm(readonly)]
        pub created_at: chrono::DateTime<Utc>,

        #[allow(unused)]
        #[lorm(updated_at)]
        #[lorm(new = "chrono::Utc::now()")]
        pub updated_at: chrono::DateTime<Utc>,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, ToLOrm)]
    pub struct Profile {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub user_id: Uuid,
        #[sqlx(json)]
        pub preferences: serde_json::Value,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    pub struct Customer {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub email: String,
        #[sqlx(flatten)]
        #[lorm(flattened(street: String, zip: String = "zip_code"))]
        pub address: Address,
    }

    #[derive(Debug, Default, Clone, lorm::ToLOrm)]
    pub struct OptCustomer {
        #[lorm(pk)]
        #[lorm(new = "Uuid::new_v4()")]
        #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub email: String,
        #[sqlx(flatten)]
        #[lorm(flattened(street: String, zip: String = "zip_code"))]
        pub address: Option<Address>,
    }

    impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for OptCustomer {
        fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
            use sqlx::Row;

            let id: Uuid = row.try_get("id")?;
            let email: String = row.try_get("email")?;
            let street: Option<String> = row.try_get("street")?;
            let zip: Option<String> = row.try_get("zip_code")?;
            let address = match (street, zip) {
                (None, None) => None,
                (Some(street), Some(zip)) => Some(Address { street, zip }),
                _ => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "partial NULL in flattened Option<Address>",
                    );
                    return Err(sqlx::Error::Decode(Box::new(err)));
                }
            };

            Ok(Self { id, email, address })
        }
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(pk_type = "manual")]
    pub struct UserRole {
        #[lorm(pk)]
        pub user_id: String,
        #[lorm(pk)]
        pub role_id: String,
        pub assigned_at: String,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(
        rename = "user_roles_named",
        pk_type = "manual",
        pk_selector = "named_by_user_role"
    )]
    pub struct UserRoleNamed {
        #[lorm(pk)]
        pub user_id: Uuid,
        #[lorm(pk)]
        pub role_name: String,
    }

    #[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
    #[lorm(pk_type = "manual")]
    pub struct Tag {
        #[lorm(pk)]
        pub name: String,
    }
}

use models::*;

#[cfg(feature = "sqlite")]
pub async fn get_pool() -> Result<Pool> {
    let database_name = Uuid::new_v4().to_string();
    let mut db_path = std::env::temp_dir();
    db_path = db_path.join(format!("{}.db", database_name));
    let database_url = format!("sqlite://{}", db_path.display());

    if Sqlite::database_exists(&database_url).await? == true {
        Sqlite::drop_database(&database_url).await?;
    }
    Sqlite::create_database(&database_url).await?;
    let pool = Pool::connect(&database_url).await?;
    let migration_path = fs::canonicalize("tests/resources/migrations/sqlite").await?;
    let mut entries: Vec<_> = Vec::new();
    let mut dir = fs::read_dir(migration_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        entries.push(entry.path());
    }
    entries.sort();
    for path in entries {
        let bytes = fs::read(&path).await?;
        let content = String::from_utf8(bytes)?;
        pool.execute(content.as_str()).await?;
    }
    Ok(pool)
}

#[cfg(feature = "postgres")]
pub async fn get_pool() -> Result<Pool> {
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://lorm:lorm@localhost:5432/lorm_test".to_string());

    let db_name = format!("lorm_test_{}", Uuid::new_v4().simple());
    let admin_pool = Pool::connect(&base_url).await?;
    admin_pool
        .execute(format!("CREATE DATABASE \"{db_name}\"").as_str())
        .await?;
    admin_pool.close().await;

    let test_url = base_url.rsplit_once('/').map_or_else(
        || format!("{base_url}/{db_name}"),
        |(base, _)| format!("{base}/{db_name}"),
    );
    let pool = Pool::connect(&test_url).await?;

    let migration_path = fs::canonicalize("tests/resources/migrations/postgres").await?;
    let mut entries: Vec<_> = Vec::new();
    let mut dir = fs::read_dir(migration_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        entries.push(entry.path());
    }
    entries.sort();
    for path in entries {
        let bytes = fs::read(&path).await?;
        let content = String::from_utf8(bytes)?;
        pool.execute(content.as_str()).await?;
    }
    Ok(pool)
}

#[cfg(feature = "mysql")]
pub async fn get_pool() -> Result<Pool> {
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://lorm:lorm@localhost:3306/lorm_test".to_string());
    let admin_url = std::env::var("DATABASE_ADMIN_URL")
        .unwrap_or_else(|_| "mysql://root:lorm@localhost:3306/lorm_test".to_string());

    let db_name = format!("lorm_test_{}", Uuid::new_v4().simple());
    let admin_pool = Pool::connect(&admin_url).await?;
    admin_pool
        .execute(format!("CREATE DATABASE `{db_name}`").as_str())
        .await?;
    admin_pool
        .execute(format!("GRANT ALL PRIVILEGES ON `{db_name}`.* TO 'lorm'@'%'").as_str())
        .await?;
    admin_pool.close().await;

    let test_url = base_url.rsplit_once('/').map_or_else(
        || format!("{base_url}/{db_name}"),
        |(base, _)| format!("{base}/{db_name}"),
    );
    let pool = Pool::connect(&test_url).await?;

    let migration_path = fs::canonicalize("tests/resources/migrations/mysql").await?;
    let mut entries: Vec<_> = Vec::new();
    let mut dir = fs::read_dir(migration_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        entries.push(entry.path());
    }
    entries.sort();
    for path in entries {
        let bytes = fs::read(&path).await?;
        let content = String::from_utf8(bytes)?;
        pool.execute(content.as_str()).await?;
    }
    Ok(pool)
}

#[tokio::test]
async fn test_user_does_not_exists() {
    let pool = get_pool().await.expect("Failed to create pool");

    let email = SafeEmail().fake::<String>();
    let res = User::by_email(&pool, &email).await;
    assert_eq!(res.is_err(), true);

    let id = Uuid::new_v4();
    let res = User::by_id(&pool, &id).await;
    assert_eq!(res.is_err(), true);
}

#[tokio::test]
async fn test_user_is_created() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = User::default();
    let email = SafeEmail().fake::<String>();
    u.email = email.clone();
    let u = u.save(&pool).await.unwrap();

    let res = User::by_id(&pool, &u.id).await;
    assert_eq!(res.is_err(), false);

    let u = res.unwrap();
    assert_eq!(u.created_at.to_rfc2822() == u.updated_at.to_rfc2822(), true);

    let res = User::by_email(&pool, &email).await;
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
    let res = User::by_id(&pool, &u.id).await;
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

    let res = User::by_id(&pool, &u.id).await;
    assert_eq!(res.is_err(), false);

    u.delete(&pool).await.unwrap();
    let res = User::by_id(&pool, &u.id).await;
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

#[cfg(feature = "sqlite")]
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
            None => format!("{i}-{email}"),
            Some(v) => format!("{v}-{email}"),
        };
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[cfg(feature = "postgres")]
async fn create_users<'e, E: sqlx::PgExecutor<'e> + Copy>(
    conn: E,
    count: i32,
    prefix: Option<&'static str>,
) -> Vec<User> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = User::default();
        u.email = match prefix {
            None => format!("{i}-{email}"),
            Some(v) => format!("{v}-{email}"),
        };
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[cfg(feature = "mysql")]
async fn create_users<'e, E: sqlx::MySqlExecutor<'e> + Copy>(
    conn: E,
    count: i32,
    prefix: Option<&'static str>,
) -> Vec<User> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = User::default();
        u.email = match prefix {
            None => format!("{i}-{email}"),
            Some(v) => format!("{v}-{email}"),
        };
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[cfg(feature = "sqlite")]
async fn create_alt_users<'e, E: sqlx::SqliteExecutor<'e> + Copy>(
    conn: E,
    count: i32,
) -> Vec<AltUser> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = AltUser::default();
        u.email = format!("{i}-{email}");
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[cfg(feature = "postgres")]
async fn create_alt_users<'e, E: sqlx::PgExecutor<'e> + Copy>(conn: E, count: i32) -> Vec<AltUser> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = AltUser::default();
        u.email = format!("{i}-{email}");
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[cfg(feature = "mysql")]
async fn create_alt_users<'e, E: sqlx::MySqlExecutor<'e> + Copy>(
    conn: E,
    count: i32,
) -> Vec<AltUser> {
    let mut users = vec![];
    for i in 0..count {
        let email = SafeEmail().fake::<String>();
        let mut u = AltUser::default();
        u.email = format!("{i}-{email}");
        u.count = Some(i);
        let u = u.save(conn).await.unwrap();
        users.push(u);
    }
    users
}

#[tokio::test]
async fn test_profile_save_with_json() {
    let pool = get_pool().await.expect("Failed to create pool");
    let profile = Profile {
        user_id: Uuid::new_v4(),
        preferences: serde_json::json!({"theme": "dark", "lang": "en"}),
        ..Default::default()
    };
    let saved = profile.save(&pool).await.unwrap();
    assert_ne!(saved.id, Uuid::nil());
    assert_eq!(saved.preferences["theme"], "dark");
}

#[tokio::test]
async fn test_profile_by_user_id_returns_json() {
    let pool = get_pool().await.expect("Failed to create pool");
    let user_id = Uuid::new_v4();
    let profile = Profile {
        user_id,
        preferences: serde_json::json!({"color": "blue"}),
        ..Default::default()
    };
    let saved = profile.save(&pool).await.unwrap();
    let fetched = Profile::by_user_id(&pool, &saved.user_id).await.unwrap();
    assert_eq!(fetched.preferences["color"], "blue");
}

#[tokio::test]
async fn test_customer_save_with_flatten() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut customer = Customer::default();
    customer.address = Address {
        street: "123 Main St".to_string(),
        zip: "90210".to_string(),
    };
    customer.email = "test@example.com".to_string();
    customer.save(&pool).await.unwrap();

    let fetched = Customer::by_email(&pool, &customer.email).await.unwrap();
    assert_eq!(fetched.address.street, "123 Main St");
    assert_eq!(fetched.address.zip, "90210");
}

#[tokio::test]
async fn test_customer_by_email_returns_flattened() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut c1 = Customer::default();
    c1.email = "alice@example.com".to_string();
    c1.address = Address {
        street: "456 Oak Ave".to_string(),
        zip: "10001".to_string(),
    };
    c1.save(&pool).await.unwrap();

    let result = Customer::by_email(&pool, &c1.email).await.unwrap();
    assert_eq!(result.address.street, "456 Oak Ave");
}

#[tokio::test]
async fn test_opt_customer_with_none_address() {
    let pool = get_pool().await.expect("Failed to create pool");
    let mut c = OptCustomer::default();
    c.email = "none@example.com".to_string();
    c.address = None;
    c.save(&pool).await.unwrap();

    let fetched = OptCustomer::by_email(&pool, &c.email).await.unwrap();
    assert!(fetched.address.is_none());
}

#[tokio::test]
async fn test_user_role_save_inserts() {
    let pool = get_pool().await.expect("Failed to create pool");

    let r = UserRole {
        user_id: Uuid::new_v4().to_string(),
        role_id: Uuid::new_v4().to_string(),
        ..Default::default()
    };
    r.save(&pool).await.unwrap();
}

#[tokio::test]
async fn test_user_role_by_key_returns_match() {
    let pool = get_pool().await.expect("Failed to create pool");

    let r = UserRole {
        user_id: Uuid::new_v4().to_string(),
        role_id: Uuid::new_v4().to_string(),
        ..Default::default()
    };
    r.save(&pool).await.unwrap();

    let fetched = UserRole::by_key(&pool, &r.user_id, &r.role_id)
        .await
        .unwrap();
    assert_eq!(fetched.user_id, r.user_id);
    assert_eq!(fetched.role_id, r.role_id);
}

#[tokio::test]
async fn test_user_role_named_by_user_role() {
    let pool = get_pool().await.expect("Failed to create pool");

    let r = UserRoleNamed {
        user_id: Uuid::new_v4(),
        role_name: "admin".to_string(),
    };
    r.save(&pool).await.unwrap();

    let fetched = UserRoleNamed::named_by_user_role(&pool, &r.user_id, &r.role_name)
        .await
        .unwrap();
    assert_eq!(fetched.user_id, r.user_id);
    assert_eq!(fetched.role_name, r.role_name);
}

#[tokio::test]
async fn test_user_role_delete_composite() {
    let pool = get_pool().await.expect("Failed to create pool");

    let r = UserRole {
        user_id: Uuid::new_v4().to_string(),
        role_id: Uuid::new_v4().to_string(),
        ..Default::default()
    };
    r.save(&pool).await.unwrap();
    r.delete(&pool).await.unwrap();

    let res = UserRole::by_key(&pool, &r.user_id, &r.role_id).await;
    assert_eq!(res.is_err(), true);
}

#[cfg(any(feature = "sqlite", feature = "postgres"))]
#[tokio::test]
async fn test_user_role_save_updates() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let ur = UserRole {
        user_id: "user_upd".to_string(),
        role_id: "editor".to_string(),
        assigned_at: "2024-01-01".to_string(),
    };
    let saved = ur.save(&pool).await.unwrap();
    assert_eq!(saved.assigned_at, "2024-01-01");

    let ur2 = UserRole {
        user_id: "user_upd".to_string(),
        role_id: "editor".to_string(),
        assigned_at: "2024-06-15".to_string(),
    };
    let upserted = ur2.save(&pool).await.unwrap();
    assert_eq!(upserted.assigned_at, "2024-06-15");

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = 'user_upd' AND role_id = 'editor'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_result, 1);

    upserted.delete(&pool).await.unwrap();
}

#[cfg(any(feature = "sqlite", feature = "postgres"))]
#[tokio::test]
async fn test_user_role_save_idempotent() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let ur = UserRole {
        user_id: "user_idem".to_string(),
        role_id: "viewer".to_string(),
        assigned_at: "2024-03-01".to_string(),
    };
    let saved1 = ur.save(&pool).await.unwrap();
    let saved2 = ur.save(&pool).await.unwrap();
    assert_eq!(saved1.assigned_at, saved2.assigned_at);

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = 'user_idem' AND role_id = 'viewer'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_result, 1);

    saved2.delete(&pool).await.unwrap();
}

#[cfg(any(feature = "sqlite", feature = "postgres"))]
#[tokio::test]
async fn test_tag_full_key_upsert_idempotent() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let tag = Tag {
        name: "rust".to_string(),
    };
    let saved1 = tag.save(&pool).await.unwrap();
    assert_eq!(saved1.name, "rust");

    let saved2 = tag.save(&pool).await.unwrap();
    assert_eq!(saved2.name, "rust");

    let count_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tags WHERE name = 'rust'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count_result, 1);

    sqlx::query("DELETE FROM tags WHERE name = 'rust'")
        .execute(&pool)
        .await
        .unwrap();
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn test_user_role_save_updates_mysql() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let ur = UserRole {
        user_id: "user_upd".to_string(),
        role_id: "editor".to_string(),
        assigned_at: "2024-01-01".to_string(),
    };
    let saved = ur.save(&pool).await.unwrap();
    assert_eq!(saved.assigned_at, "2024-01-01");

    let ur2 = UserRole {
        user_id: "user_upd".to_string(),
        role_id: "editor".to_string(),
        assigned_at: "2024-06-15".to_string(),
    };
    let upserted = ur2.save(&pool).await.unwrap();
    assert_eq!(upserted.assigned_at, "2024-06-15");

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = 'user_upd' AND role_id = 'editor'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_result, 1);

    upserted.delete(&pool).await.unwrap();
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn test_user_role_save_idempotent_mysql() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let ur = UserRole {
        user_id: "user_idem".to_string(),
        role_id: "viewer".to_string(),
        assigned_at: "2024-03-01".to_string(),
    };
    let saved1 = ur.save(&pool).await.unwrap();
    let saved2 = ur.save(&pool).await.unwrap();
    assert_eq!(saved1.assigned_at, saved2.assigned_at);

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = 'user_idem' AND role_id = 'viewer'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_result, 1);

    saved2.delete(&pool).await.unwrap();
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn test_tag_full_key_upsert_idempotent_mysql() {
    use models::*;
    let pool = get_pool().await.expect("Failed to create pool");

    let tag = Tag {
        name: "rust".to_string(),
    };
    let saved1 = tag.save(&pool).await.unwrap();
    assert_eq!(saved1.name, "rust");

    let saved2 = tag.save(&pool).await.unwrap();
    assert_eq!(saved2.name, "rust");

    let count_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tags WHERE name = 'rust'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count_result, 1);

    sqlx::query("DELETE FROM tags WHERE name = 'rust'")
        .execute(&pool)
        .await
        .unwrap();
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_self_ref_category_compiles() {
    use models::Category;

    let pool = get_pool().await.expect("Failed to create pool");

    let mut cat = Category::default();
    cat.name = "Root".to_string();
    let cat = cat.save(&pool).await.expect("Category save failed");

    let parent = cat.parent();
    assert!(parent.is_none(), "Expected parent() to return None");

    let children = cat
        .children()
        .build(&pool)
        .await
        .expect("children() build failed");
    assert!(children.is_empty(), "Expected no children");
}

#[tokio::test]
async fn test_backcompat_generated_pk_by_id_still_works() {
    let pool = get_pool().await.expect("Failed to create pool");

    let mut u = User::default();
    u.email = SafeEmail().fake::<String>();
    let u = u.save(&pool).await.unwrap();

    let fetched = User::by_id(&pool, &u.id).await.unwrap();
    assert_eq!(fetched.id, u.id);
}

#[cfg(feature = "sqlite")]
mod relations {
    use super::get_pool;
    use super::models::*;

    #[tokio::test]
    async fn test_belongs_to_basic() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut u = User::default();
        u.email = "belongs-to@test.com".to_string();
        let u = u.save(&pool).await.unwrap();

        let mut p = Post::default();
        p.title = "Hello World".to_string();
        p.user_id = u.id;
        let p = p.save(&pool).await.unwrap();

        let users = p.user().build(&pool).await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, u.id);
    }

    #[tokio::test]
    async fn test_has_many_returns_all() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut u = User::default();
        u.email = "has-many@test.com".to_string();
        let u = u.save(&pool).await.unwrap();

        for i in 0..3_u32 {
            let mut p = Post::default();
            p.title = format!("Post {i}");
            p.user_id = u.id;
            p.save(&pool).await.unwrap();
        }

        let posts = u.posts().build(&pool).await.unwrap();
        assert_eq!(posts.len(), 3);
    }

    #[tokio::test]
    async fn test_has_one_semantics() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut u = User::default();
        u.email = "has-one@test.com".to_string();
        let u = u.save(&pool).await.unwrap();

        let profile = Profile {
            user_id: u.id,
            preferences: serde_json::json!({"theme": "light"}),
            ..Default::default()
        };
        let profile = profile.save(&pool).await.unwrap();

        let profile_opt = u
            .profile()
            .limit(1)
            .build(&pool)
            .await
            .unwrap()
            .into_iter()
            .next();
        assert!(profile_opt.is_some());
        assert_eq!(profile_opt.unwrap().id, profile.id);
    }

    #[tokio::test]
    async fn test_fk_override() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut parent = Category::default();
        parent.name = "Parent".to_string();
        let parent = parent.save(&pool).await.unwrap();

        let mut child = Category::default();
        child.name = "Child".to_string();
        child.parent_id = Some(parent.id);
        let child = child.save(&pool).await.unwrap();

        let children = parent.children().build(&pool).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child.id);
    }

    #[tokio::test]
    async fn test_method_name_override() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut cat = Category::default();
        cat.name = "Lone Category".to_string();
        let cat = cat.save(&pool).await.unwrap();

        let children = cat.children().build(&pool).await.unwrap();
        assert!(children.is_empty());
    }

    #[tokio::test]
    async fn test_self_ref_tree() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut root = Category::default();
        root.name = "Root".to_string();
        let root = root.save(&pool).await.unwrap();

        for i in 0..2_u32 {
            let mut c = Category::default();
            c.name = format!("Child {i}");
            c.parent_id = Some(root.id);
            c.save(&pool).await.unwrap();
        }

        let children = root.children().build(&pool).await.unwrap();
        assert_eq!(children.len(), 2);

        let child = &children[0];
        let parent_builder = child.parent().expect("child should have a parent builder");
        let parents = parent_builder.build(&pool).await.unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].id, root.id);
    }

    #[tokio::test]
    async fn test_composability() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut u = User::default();
        u.email = "compose@test.com".to_string();
        let u = u.save(&pool).await.unwrap();

        for i in 0..5_u32 {
            let mut p = Post::default();
            p.title = format!("Post {i}");
            p.user_id = u.id;
            p.save(&pool).await.unwrap();
        }

        let posts = u.posts().limit(3).build(&pool).await.unwrap();
        assert_eq!(posts.len(), 3);
    }

    #[tokio::test]
    async fn test_nullable_fk_belongs_to() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut d = Draft::default();
        d.title = "Untitled Draft".to_string();
        let d = d.save(&pool).await.unwrap();

        let user_builder = d.user();
        assert!(user_builder.is_none());
    }

    #[tokio::test]
    async fn test_multiple_relations_one_struct() {
        let pool = get_pool().await.expect("Failed to create pool");

        let mut u = User::default();
        u.email = "multi-rel@test.com".to_string();
        let u = u.save(&pool).await.unwrap();

        let mut p = Post::default();
        p.title = "My Post".to_string();
        p.user_id = u.id;
        p.save(&pool).await.unwrap();

        let profile = Profile {
            user_id: u.id,
            preferences: serde_json::json!({}),
            ..Default::default()
        };
        profile.save(&pool).await.unwrap();

        let posts = u.posts().build(&pool).await.unwrap();
        assert_eq!(posts.len(), 1);

        let profiles = u.profile().build(&pool).await.unwrap();
        assert_eq!(profiles.len(), 1);
    }
}
