use std::{env, str};
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
  routes::*,
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

  let data = sled::open("./data").unwrap();

  if data.contains_key("source_ssid").unwrap() {
    let ssid = data.get("source_ssid").unwrap().unwrap();
    let pwd = data.get("source_password").unwrap().unwrap();

    let str_ssid = str::from_utf8(ssid.as_ref()).unwrap();
    let str_pwd = str::from_utf8(pwd.as_ref()).unwrap();

    run_command(&format!("nmcli dev wifi connect {str_ssid} password \"{str_pwd}\""));
  }

  run_command("nmcli con modify Hotspot wifi-sec.pmf disable");
  run_command("iptables -t nat -A PREROUTING -i wlan0 -p tcp --dport 80 -j REDIRECT --to-port 8000");

  rocket::build()
    .attach(Template::fairing())
    .register("/", catchers![login])
    .mount("/", FileServer::from(relative!("static")))
    .mount("/", routes![
      index,
      auth,
      status_page,
      wireless,
      save_wireless,
      credential,
      save_credential,
      restart,
      logout,
    ],
  )
}
