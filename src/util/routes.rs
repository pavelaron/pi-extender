use std::str;

use local_ip_address::{list_afinet_netifas, local_ip};
use rocket::{form::Form, http::{Cookie, CookieJar, Status}, response::Redirect, Request};
use rocket_dyn_templates::{Template, context};
use sysinfo::System;

use crate::util::{
  structs::{AuthenticatedUser, LoginInput},
  output_utils::{format_ferris, run_command},
  crypto_utils::{generate_token, hash_password, get_salt},
};

#[get("/")]
pub fn index(_user: AuthenticatedUser) -> Template {
  Template::render("index", context! {})
}

#[catch(401)]
pub fn login(_r: &Request) -> Template {
  Template::render("login", context! { message: "FAIL" })
}

#[get("/status")]
pub fn status_page(_user: AuthenticatedUser) -> Template {
  let system = System::new_all();

  let response_str = format_ferris("Repeater is up and running!");
  let boot = System::boot_time();
  let load_avg = System::load_average();
  let free_mem = system.free_memory();
  let hostname = System::host_name();
  let total_mem = system.total_memory();
  let ip = local_ip().unwrap();
  let network_interfaces: String = list_afinet_netifas()
    .unwrap()
    .iter()
    .fold(
      String::from(""), 
      |acc: String, (name, ip)| format!("{}{}:\t{:?}\n", acc, name, ip),
    );

  Template::render("status", context! {
    response: response_str,
    boot_time: format!("{}", boot),
    load_avg: format!("{:?}%", load_avg.five),
    memory: format!("{:?} of {:?} bytes", free_mem, total_mem),
    hostname: hostname,
    local_ip: ip,
    interfaces: network_interfaces,
  })
}

#[get("/user/profile")]
pub fn profile(user: AuthenticatedUser) -> String {
    // Only accessible with valid JWT token
    format!("Profile for user: {}", user.user_id)
}

#[post("/", data = "<login_input>")]
pub fn auth(login_input: Form<LoginInput>, cookies: &CookieJar<'_>) -> Redirect {
  let username = &login_input.username;
  let password = &login_input.password;

  let data = sled::open("./data").unwrap();
  let salt = &get_salt(&data);

  if !data.contains_key(username).unwrap() {
    data
      .insert("admin", hash_password("changeme", salt).as_str())
      .expect("Failed to save default credentials");

    let _ = data.flush();
  }

  let stored = data.get(username).unwrap().unwrap();
  let val = str::from_utf8(stored.as_ref()).unwrap();

  if val != hash_password(password, salt) {
    return Redirect::to("/");
  }

  let token_string = match generate_token(username) {
    Ok(token) => Ok(token),
    Err(_) => Err(Status::InternalServerError),
  };

  cookies.add(Cookie::new("token", token_string.unwrap()));

  Redirect::to("/")
}

#[get("/logout")]
pub fn logout(_user: AuthenticatedUser, cookies: &CookieJar<'_>) -> Redirect {
  cookies.remove(Cookie::from("token"));
  Redirect::to("/")
}

#[get("/restart")]
pub fn restart(_user: AuthenticatedUser) -> Redirect {
  run_command("reboot -h now");
  Redirect::to("/")
}
