use super::schema::posts;
use serde::Serialize;

#[derive(Serialize, Queryable)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}
