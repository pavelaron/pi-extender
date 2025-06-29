use std::{
  collections::HashMap, net::IpAddr, os::unix::process::ExitStatusExt, process::{Command, Stdio}, str
};
use local_ip_address::{list_afinet_netifas, local_ip};
use rocket::{
  form::Form,
  http::{Cookie, CookieJar, Status},
  response::{content::RawHtml, Redirect},
  Request,
  State,
};

use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsContextManager, HandlebarsResponse};
use sled::{Db, Batch, IVec, open};
use sysinfo::System;

use crate::util::wireless_utils::{
  connect_to_network,
  disable_pwr_mgmt,
  get_interfaces,
};

use super::{
  crypto_utils::{generate_token, get_salt, hash_password},
  output_utils::{format_ferris, get_pwa_headers, run_command},
  structs::{AuthenticatedUser, LoginInput, WirelessInput},
};

#[get("/")]
pub fn index(
  user: AuthenticatedUser,
  handlebars_cm: &State<HandlebarsContextManager>,
  etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
  let headers: String = user.pwa_headers.to_string();
  
  let map: HashMap<&str, String> = HashMap::from([
    ("pwa_headers", headers),
  ]);

  handlebars_response!(handlebars_cm, etag_if_none_match, "index", map)
}

#[catch(401)]
pub fn login(req: &Request) -> RawHtml<String> {
  let context_manager: Option<&HandlebarsContextManager> = req.rocket().state::<HandlebarsContextManager>();
  let map: HashMap<&str, String> = HashMap::from([
    ("pwa_headers", get_pwa_headers(req)),
  ]);

  let rendered_html: String = context_manager.unwrap().render("login", map);

  RawHtml(rendered_html)
}

#[catch(default)]
pub fn default_error(status: Status, req: &Request) -> RawHtml<String> {
  let context_manager: &HandlebarsContextManager = req.rocket().state::<HandlebarsContextManager>().unwrap();
  let context: HashMap<&'static str, String> = HashMap::from([
    ("pwa_headers", get_pwa_headers(req)),
    ("status",      status.code.to_string()),
    ("message",     format_ferris(&status.reason().unwrap())),
  ]);

  let rendered_html: String = context_manager.render("error", context);

  RawHtml(rendered_html)
}

#[get("/.well-known/appspecific/com.chrome.devtools.json")]
pub fn devtools() -> RawHtml<String> {
  RawHtml(String::from("{\"name\": \"Pi Extender\"}"))
}

#[get("/status")]
pub fn status_page(
  user: AuthenticatedUser,
  handlebars_cm: &State<HandlebarsContextManager>,
  etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
  let system = System::new_all();

  let response_str = format_ferris("Repeater is up and running!");
  let boot = System::boot_time();
  let cpu_usage = system.global_cpu_usage();
  let free_mem = system.free_memory();
  let hostname = System::host_name();
  let total_mem = system.total_memory();
  let ip = local_ip().unwrap();
  let network_interfaces: Vec<(String, IpAddr)> = list_afinet_netifas().unwrap();

  let output = Command::new("iwgetid")
    .output()
    .unwrap_or_else(|_| {
      std::process::Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: "Not available".as_bytes().to_vec(),
        stderr: Vec::new(),
      }
    });

  drop(system);

  let source_ap = String::from_utf8(output.stdout).unwrap();

  let status_data = serde_json::json! {{
    "pwa_headers": user.pwa_headers,
    "response": response_str,
    "boot_time": format!("{}", boot),
    "cpu_usage": format!("{:?}%", cpu_usage),
    "memory": format!("{:?} of {:?} bytes", free_mem, total_mem),
    "source_ap": source_ap,
    "hostname": hostname.unwrap(),
    "local_ip": ip.to_string(),
    "interfaces": network_interfaces,
  }};

  handlebars_response!(handlebars_cm, etag_if_none_match, "status", status_data)
}

#[get("/wireless-settings")]
pub fn wireless(
  user: AuthenticatedUser,
  handlebars_cm: &State<HandlebarsContextManager>,
  etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
  let data: Db = open("./data").unwrap();
  let interfaces: Vec<String> = get_interfaces();

  let mut json: serde_json::Value = serde_json::json! {{
    "pwa_headers": user.pwa_headers,
    "interfaces":  interfaces,
  }};

  for item in data.scan_prefix(b"ap_") {
    if item.is_err() {
      continue;
    }

    let (item_key, item_value) = item.unwrap();

    let key: String = String::from_utf8(item_key.to_vec()).unwrap();
    let value: String = String::from_utf8(item_value.to_vec()).unwrap();

    json[key] = serde_json::Value::String(value);
  }

  drop(data);

  handlebars_response!(handlebars_cm, etag_if_none_match, "wireless", json)
}

#[get("/credential-settings")]
pub fn credential(
  user: AuthenticatedUser,
  handlebars_cm: &State<HandlebarsContextManager>,
  etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
  let map: HashMap<&str, String> = HashMap::from([
    ("username",      user.user_id),
    ("pwa_headers",   user.pwa_headers),
  ]);

  handlebars_response!(handlebars_cm, etag_if_none_match, "credential", map)
}

#[post["/save-wireless", data = "<wireless_input>"]]
pub fn save_wireless(wireless_input: Form<WirelessInput>) -> Redirect {
  let data: Db = open("./data").unwrap();
  let mut batch: Batch = Batch::default();

  let src_ssid: &str = wireless_input.source_ssid.as_str();
  let src_password: &str = wireless_input.source_password.as_str();

  batch.insert("ap_ssid", wireless_input.ap_ssid.as_str());
  batch.insert("ap_password", wireless_input.ap_password.as_str());
  batch.insert("ap_interface", wireless_input.ap_interface.as_str());

  data
    .apply_batch(batch)
    .expect("Failed to save wireless settings");

  let _ = data.flush();

  connect_to_network(src_ssid, src_password);
  disable_pwr_mgmt(&wireless_input.ap_interface);

  drop(data);

  let _ = Command::new("sh")
    .arg("-c")
    .arg("sleep 5 ; reboot")
    .stdin(Stdio::null())
    .stdout(Stdio::null()) 
    .stderr(Stdio::null())
    .spawn();

  Redirect::to("/")
}

#[post["/save-credential", data = "<credential_input>"]]
pub fn save_credential(
  credential_input: Form<LoginInput>,
  cookies: &CookieJar<'_>,
) -> Redirect {
  let data: Db = open("./data").unwrap();
  let mut batch: Batch = Batch::default();

  let username: &str = credential_input.username.as_str();
  let password: &str = credential_input.password.as_str();

  batch.insert("username", username);
  batch.insert("password", password);

  data
    .apply_batch(batch)
    .expect("Failed to save credential settings");

  let _ = data.flush();
  drop(data);

  let token_string: Result<String, Status> = match generate_token(username) {
    Ok(token) => Ok(token),
    Err(_) => Err(Status::InternalServerError),
  };

  cookies.add(Cookie::new("token", token_string.unwrap()));

  Redirect::to("/")
}

#[post("/", data = "<login_input>")]
pub fn auth(login_input: Form<LoginInput>, cookies: &CookieJar<'_>) -> Redirect {
  let username: &String = &login_input.username;
  let password: &String = &login_input.password;

  let data: Db = open("./data").unwrap();
  let salt: &String = &get_salt(&data);

  if !data.contains_key(username).unwrap() {
    data
      .insert("admin", hash_password("changeme", salt).as_str())
      .expect("Failed to save default credentials");

    let _ = data.flush();
  }

  let stored: IVec = data.get(username).unwrap().unwrap();
  let val: &str = str::from_utf8(stored.as_ref()).unwrap();

  drop(data);

  if val != hash_password(password, salt) {
    return Redirect::to("/");
  }

  let token_string: Result<String, Status> = match generate_token(username) {
    Ok(token) => Ok(token),
    Err(_) => Err(Status::InternalServerError),
  };

  cookies.add(Cookie::new("token", token_string.unwrap()));

  Redirect::to("/")
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
  cookies.remove(Cookie::from("token"));
  Redirect::to("/")
}

#[get("/restart")]
pub fn restart() -> Redirect {
  run_command("reboot", &["-h", "now"]);
  Redirect::to("/")
}
