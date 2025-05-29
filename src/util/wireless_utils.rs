use std::str;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use super::output_utils::run_command;

pub fn initialize_wireless() {
  let network_interfaces = NetworkInterface::show().unwrap(); // ::show().unwrap();

  for itf in network_interfaces.iter() {
    println!("{:?}", itf);
  }

  let data = sled::open("./data").unwrap();

  if !data.contains_key("source_ssid").unwrap() {
    run_command("nmcli", &[
      "d",
      "wifi",
      "hotspot",
      "ifname",
      "wlan0",
      "ssid",
      "pi-extender",
      "password",
      "changeme",
    ]);

    return;
  }

  let ssid = data.get("source_ssid").unwrap().unwrap();
  let pwd = data.get("source_password").unwrap().unwrap();

  let str_ssid = str::from_utf8(ssid.as_ref()).unwrap();
  let str_pwd = str::from_utf8(pwd.as_ref()).unwrap();

  // nmcli --get-values GENERAL.DEVICE,GENERAL.TYPE device show | awk '/^wifi/{print dev; next};{dev=$0};'

  run_command("nmcli", &["con", "modify", "Hotspot", "wifi-sec.pmf", "disable"]);
  run_command("nmcli", &[
    "dev",
    "wifi",
    "connect",
    &format!("\"{str_ssid}\""),
    "password",
    &format!("\"{str_pwd}\""),
  ]);
}
