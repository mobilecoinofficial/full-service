pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::Connection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use std::env;

use models::*;
use posts::dsl::posts as dsl_posts;
use posts::published;
use schema::posts;

use uuid::Uuid;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_post(conn: &SqliteConnection, title: &str, body: &str) -> String {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();

    let new_post = NewPost {
        id: &uuid,
        title: title,
        body: body,
    };

    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new post");

    uuid
}

pub fn publish_post(conn: &SqliteConnection, key: String) -> usize {
    diesel::update(dsl_posts.find(key))
        .set(published.eq(true))
        .execute(conn)
        .expect("Unable to find post")
}
