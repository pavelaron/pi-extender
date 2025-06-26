use std::str;
use sled::IVec;
use waifai::{Hotspot, WiFi};

use super::output_utils::run_command;

pub fn initialize_wireless() {
  let data = sled::open("./data").unwrap();
  
  initialize_ap(&data);
  disable_pwr_mgmt("wlan0");

  drop(data);
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
  match WiFi::interfaces () {
    Ok(interfaces) => interfaces,
    Err(error) => {
      println!("Error retrieving interfaces: {}", error);
      Vec::new()
    }
  }
}

pub fn disable_pwr_mgmt(interface: &str) {
  run_command("iwconfig", &[
    interface,
    "power",
    "off",
  ]);
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
  
  let ap_interface: String = match stored_interface {
    Some(stored_interface) => String::from_utf8(stored_interface.to_vec()).unwrap(),
    None => String::from("wlan0"),
  };

  let ssid = str::from_utf8(stored_ssid.as_ref()).unwrap();
  let password = str::from_utf8(stored_password.as_ref()).unwrap();

  let hotspot = WiFi::new(ap_interface);

  match hotspot.create(ssid, Some(password)) {
    Ok(instance) => {
      let _ = hotspot.start();
      disable_pwr_mgmt(hotspot.interface());
      instance
    },
    Err(error) => {
      println!("Error creating hotspot: {}", error);
      return;
    },
  };
}
