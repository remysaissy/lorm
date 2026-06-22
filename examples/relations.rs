//! Relations example
//!
//! This example demonstrates supported relations:
//! - belongs_to
//! - has_many
//! - has_one
//! - self-referential relations

use anyhow::Result;
use lorm::ToLOrm;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
#[lorm(has_many = Post)]
#[lorm(has_one = Profile)]
pub struct User {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub name: String,
}

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
pub struct Post {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,

    pub title: String,

    #[lorm(belongs_to = User)]
    pub user_id: Uuid,
}

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
pub struct Profile {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,

    pub bio: String,

    #[lorm(belongs_to = User)]
    pub user_id: Uuid,
}

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
#[lorm(has_many(Self, fk = "parent_id", as = "children"))]
pub struct Category {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub name: String,

    #[lorm(belongs_to = Self)]
    pub parent_id: Option<Uuid>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an in-memory database
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        );
        CREATE TABLE posts (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            user_id TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );
        CREATE TABLE profiles (
            id TEXT PRIMARY KEY NOT NULL,
            bio TEXT NOT NULL,
            user_id TEXT NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );
        CREATE TABLE categories (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            parent_id TEXT,
            FOREIGN KEY(parent_id) REFERENCES categories(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    println!("=== Relations Example ===\n");

    // 1. Setup Data
    println!("1. Setting up data...");
    let mut alice = User {
        name: "Alice".to_string(),
        ..Default::default()
    };
    alice = alice.save(&pool).await?;

    let mut post1 = Post {
        title: "Hello Lorm".to_string(),
        user_id: alice.id,
        ..Default::default()
    };
    post1 = post1.save(&pool).await?;

    let post2 = Post {
        title: "Advanced Relations".to_string(),
        user_id: alice.id,
        ..Default::default()
    };
    let _post2 = post2.save(&pool).await?;

    let profile = Profile {
        bio: "Rust Enthusiast".to_string(),
        user_id: alice.id,
        ..Default::default()
    };
    let _profile = profile.save(&pool).await?;

    let mut parent_cat = Category {
        name: "Tech".to_string(),
        ..Default::default()
    };
    parent_cat = parent_cat.save(&pool).await?;

    let mut child_cat = Category {
        name: "Rust".to_string(),
        parent_id: Some(parent_cat.id),
        ..Default::default()
    };
    child_cat = child_cat.save(&pool).await?;
    println!("   Setup complete.\n");

    // 2. Demonstrate has_many
    println!("2. Demonstrating has_many (User -> Posts)...");
    let posts = alice.posts().build(&pool).await?;
    println!("   User {} has {} posts:", alice.name, posts.len());
    for post in posts {
        println!("     - {}", post.title);
    }
    println!();

    // 3. Demonstrate belongs_to
    println!("3. Demonstrating belongs_to (Post -> User)...");
    let post_author = post1.user().build(&pool).await?.into_iter().next().unwrap();
    println!(
        "   Post '{}' belongs to user: {}\n",
        post1.title, post_author.name
    );

    // 4. Demonstrate has_one
    println!("4. Demonstrating has_one (User -> Profile)...");
    let user_profile = alice
        .profile()
        .limit(1)
        .build(&pool)
        .await?
        .into_iter()
        .next();
    if let Some(p) = user_profile {
        println!("   User {} has bio: {}\n", alice.name, p.bio);
    }

    // 5. Demonstrate self-referential relations
    println!("5. Demonstrating self-referential relations (Category)...");
    let parent = child_cat
        .parent()
        .unwrap()
        .build(&pool)
        .await?
        .into_iter()
        .next()
        .unwrap();
    println!(
        "   Category '{}' has parent: {}",
        child_cat.name, parent.name
    );

    let children = parent_cat.children().build(&pool).await?;
    println!(
        "   Category '{}' has {} children:",
        parent_cat.name,
        children.len()
    );
    for child in children {
        println!("     - {}", child.name);
    }
    println!();

    println!(
        "Summary: Lorm provides explicit composable relations. Queries are executed only when .build() is called."
    );

    Ok(())
}
