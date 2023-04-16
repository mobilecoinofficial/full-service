use mc_transaction_signer::service::api::api;

use rocket::{self, launch, routes, Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![api])
}
