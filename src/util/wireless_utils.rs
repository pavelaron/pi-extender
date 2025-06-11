use std::str;
use sled::IVec;

use super::output_utils::run_command;

pub fn initialize_wireless() {
  let data = sled::open("./data").unwrap();
  
  initialize_ap(&data);
}

pub fn connect_to_network(ssid: &str, password: &str) {
  run_command("nmcli", &[
    "dev",
    "wifi",
    "connect",
    &format!("\"{ssid}\""),
    "password",
    &format!("\"{password}\""),
  ]);
}

pub fn get_interfaces() -> Vec<String> {
  let raw_interfaces = run_command("nmcli", &[
    "-t",
    "-f",
    "active,wifi",
  ]);

  if raw_interfaces.is_none() {
    return Vec::new();
  }

  let nmcli_result = String::from_utf8(raw_interfaces.unwrap().stdout);

  let interfaces: Vec<String> = nmcli_result
    .unwrap()
    .lines()
    .filter(|line| {
      line.contains(": connected") || line.contains(": disconnected")
    })
    .filter_map(|line| line.split(':').next())
    .map(|line| line.to_string())
    .collect();

  interfaces
}

fn initialize_ap(data: &sled::Db) {
  let stored_ssid = data.get("ap_ssid")
    .unwrap()
    .unwrap_or(IVec::from("pi-extender"));
  
  let stored_password = data.get("ap_password")
    .unwrap()
    .unwrap_or(IVec::from("changeme"));

  let ap_interface_binding = data
    .get("ap_interface")
    .unwrap();

  let stored_interface = ap_interface_binding
    .as_ref();

  let ifname: &[&str] = match stored_interface {
    Some(stored_interface) => &[
      "ifname",
      str::from_utf8(stored_interface).unwrap(),
    ],
    None => &[],
  };

  let args = [
    "device",
    "wifi",
    "hotspot",
  ]
  .iter()
  .chain(ifname.iter())
  .chain([
    "ssid",
    str::from_utf8(stored_ssid.as_ref()).unwrap(),
    "password",
    str::from_utf8(stored_password.as_ref()).unwrap(),
  ].iter())
  .copied()
  .collect::<Vec<_>>();

  run_command("nmcli", args.as_slice());
}
