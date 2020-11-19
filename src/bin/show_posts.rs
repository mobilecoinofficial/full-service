extern crate diesel;
extern crate mc_wallet_service;

use diesel::prelude::*;
use mc_wallet_service::models::*;

fn main() {
    use mc_wallet_service::establish_connection;
    use mc_wallet_service::schema::posts::dsl::*;

    let connection = establish_connection();
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }
}
