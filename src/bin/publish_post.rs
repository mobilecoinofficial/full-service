extern crate diesel;
extern crate mc_wallet_service;

use self::mc_wallet_service::*;
use std::env::args;

fn main() {
    let id = args().nth(1).expect("publish_post requires a post id");
    let connection = establish_connection();

    let update_int = publish_post(&connection, id);
    println!("Published post {}", update_int);
}
