use std::{env, str};
use dotenv::dotenv;
use rocket::{
  http::{Cookie, Status},
  request::{FromRequest, Outcome, Request},
  Build,
  Rocket,
};
use rocket_include_dir::{include_dir, Dir, StaticFiles};
use rocket_include_handlebars::HandlebarsResponse;
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
    output,
    get_pwa_headers,
  },
  wireless_utils::initialize_wireless,
  routes::*,
};

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_handlebars;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let cookies = request.cookies().get("token");
    let pwa_headers = get_pwa_headers(request);

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
        if token_data.claims.exp < Utc::now().timestamp() {
          return Outcome::Error((Status::Unauthorized, ()));
        }

        let refreshed_token = match generate_token(&token_data.claims.sub) {
          Ok(token) => token,
          Err(error) => {
            println!("Token generation error: {}", error);
            return Outcome::Error((Status::Unauthorized, ()));
          }
        };

        request
          .cookies()
          .add(Cookie::new("token", refreshed_token));

        Outcome::Success(AuthenticatedUser {
          user_id: token_data.claims.sub,
          pwa_headers,
        })
      },
      Err(error) => {
        println!("Error decoding token: {}", error);
        Outcome::Error((Status::Unauthorized, ()))
      }
    }
  }
}

#[launch]
fn launch() -> Rocket<Build> {
  output("Initializing admin server...");
  dotenv().ok();

  initialize_wireless();

  static STATIC_DIR: Dir = include_dir!("static");

  rocket::build()
    .attach(HandlebarsResponse::fairing(|handlebars| {
      handlebars_resources_initialize!(
          handlebars,
          "index"       => "templates/index.html.hbs",
          "wireless"    => "templates/wireless.html.hbs",
          "error"       => "templates/error.html.hbs",
          "login"       => "templates/login.html.hbs",
          "status"      => "templates/status.html.hbs",
          "credential"  => "templates/credential.html.hbs",
          "pwa"         => "templates/headers.hbs",
        );
      })
    )
    .register("/", catchers![
      login,
      default_error,
    ])
    .mount("/", StaticFiles::from(&STATIC_DIR))
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
      devtools,
    ],
  )
}
