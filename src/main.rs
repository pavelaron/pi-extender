use std::env;
use dotenv::dotenv;
use rocket::{
  fs::{relative, FileServer},
  http::{Cookie, Status},
  request::{FromRequest, Outcome, Request},
  Build,
  Rocket,
};
use rocket_dyn_templates::Template;
use chrono::Utc;
use jsonwebtoken::{
  decode,
  DecodingKey,
  Validation,
};

mod util;

use crate::util::{
  structs::{
    AuthenticatedUser,
    Claims,
  },
  crypto_utils::generate_token,
  output_utils::{
    run_command,
    output,
  },
  routes,
};

#[macro_use]
extern crate rocket;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let cookies = request.cookies().get("token");

    let token = match cookies {
      Some(token) => token.value(),
      None => return Outcome::Error((Status::Unauthorized, ())),
    };

    // Validate token
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::default();

    match decode::<Claims>(
      &token,
      &DecodingKey::from_secret(secret.as_bytes()),
      &validation,
    ) {
      Ok(token_data) => {
        println!("Token data: {:#?}", token_data);

        if token_data.claims.exp < Utc::now().timestamp() {
          return Outcome::Error((Status::Unauthorized, ()));
        }

        let refreshed_token = match generate_token(&token_data.claims.sub) {
          Ok(token) => token,
          Err(error) => {
            println!("Error: {}", error);
            return Outcome::Error((Status::Unauthorized, ()));
          }
        };

        request
          .cookies()
          .add(Cookie::new("token", refreshed_token));

        Outcome::Success(AuthenticatedUser {
          user_id: token_data.claims.sub,
        })
      },
      Err(error) => {
        println!("Error: {}", error);
        Outcome::Error((Status::Unauthorized, ()))
      }
    }
  }
}

#[launch]
fn launch() -> Rocket<Build> {
  output("Initializing admin server...");
  dotenv().ok();

  run_command("nmcli con modify Hotspot wifi-sec.pmf disable");
  run_command("iptables -t nat -A PREROUTING -i wlan0 -p tcp --dport 80 -j REDIRECT --to-port 8000");

  rocket::build()
    .attach(Template::fairing())
    .register("/", catchers![routes::login])
    .mount("/", FileServer::from(relative!("static")))
    .mount("/", routes![
      routes::index,
      routes::auth,
      routes::status_page,
      routes::wireless,
      routes::save_wireless,
      routes::restart,
      routes::logout,
    ],
  )
}
