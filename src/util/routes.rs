use std::str;

use local_ip_address::{list_afinet_netifas, local_ip};
use rocket::{
  form::Form,
  http::{Cookie, CookieJar, Status},
  response::Redirect,
  Request,
};
use rocket_dyn_templates::{Template, context};
use sysinfo::System;

use super::{
  structs::{AuthenticatedUser, LoginInput, WirelessInput},
  output_utils::{format_ferris, run_command, error_context},
  crypto_utils::{generate_token, hash_password, get_salt},
};

#[get("/")]
pub fn index(_user: AuthenticatedUser) -> Template {
  Template::render("index", context! {})
}

#[catch(401)]
pub fn login(_r: &Request) -> Template {
  Template::render("login", context! {})
}

#[catch(404)]
pub fn not_found(_r: &Request) -> Template {
  Template::render("error", error_context(404, "Page not found"))
}

#[catch(500)]
pub fn internal_error(_r: &Request) -> Template {
  Template::render("error", error_context(500, "Internal server error"))
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

#[get("/wireless-settings")]
pub fn wireless(_user: AuthenticatedUser) -> Template {
  let data = sled::open("./data").unwrap();

  if data.contains_key("source_ssid").unwrap() {
    let source_ssid = data.get("source_ssid").unwrap().unwrap();
    let source_password = data.get("source_password").unwrap().unwrap();
    let ap_ssid = data.get("ap_ssid").unwrap().unwrap();
    let ap_password = data.get("ap_password").unwrap().unwrap();

    return Template::render("wireless", context! {
      source_ssid: str::from_utf8(source_ssid.as_ref()).unwrap(),
      source_password: str::from_utf8(source_password.as_ref()).unwrap(),
      ap_ssid: str::from_utf8(ap_ssid.as_ref()).unwrap(),
      ap_password: str::from_utf8(ap_password.as_ref()).unwrap(),
    });
  }

  Template::render("wireless", context! {})
}

#[get("/credential-settings")]
pub fn credential(user: AuthenticatedUser) -> Template {
  Template::render("credential", context! {
    username: user.user_id,
  })
}

#[post["/save-wireless", data = "<wireless_input>"]]
pub fn save_wireless(_user: AuthenticatedUser, wireless_input: Form<WirelessInput>) -> Redirect {
  let data = sled::open("./data").unwrap();
  let mut batch = sled::Batch::default();

  batch.insert("source_ssid", wireless_input.source_ssid.as_str());
  batch.insert("source_password", wireless_input.source_password.as_str());
  batch.insert("ap_ssid", wireless_input.ap_ssid.as_str());
  batch.insert("ap_password", wireless_input.ap_password.as_str());

  data
    .apply_batch(batch)
    .expect("Failed to save wireless settings");

  let _ = data.flush();

  run_command("reboot", &["-h", "now"]);

  Redirect::to("/")
}

#[post["/save-credential", data = "<credential_input>"]]
pub fn save_credential(
  _user: AuthenticatedUser,
  credential_input: Form<LoginInput>,
  cookies: &CookieJar<'_>,
) -> Redirect {
  let data = sled::open("./data").unwrap();
  let mut batch = sled::Batch::default();

  let username = credential_input.username.as_str();
  let password = credential_input.password.as_str();

  batch.insert("username", username);
  batch.insert("password", password);

  data
    .apply_batch(batch)
    .expect("Failed to save credential settings");

  let _ = data.flush();

  let token_string = match generate_token(username) {
    Ok(token) => Ok(token),
    Err(_) => Err(Status::InternalServerError),
  };

  cookies.add(Cookie::new("token", token_string.unwrap()));

  Redirect::to("/")
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
  run_command("reboot", &["-h", "now"]);
  Redirect::to("/")
}
