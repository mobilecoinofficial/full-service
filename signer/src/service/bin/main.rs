use mc_signer::service::api::signer_service_api;

use rocket::{self, launch, routes, Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![signer_service_api])
}
