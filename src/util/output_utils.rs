use ferris_says::say;
use rocket::{response::content::RawHtml, Request};
use rocket_include_handlebars::HandlebarsContextManager;
use std::{
  collections::HashMap,
  io::{stdout, BufWriter},
  process::Command,
};

pub fn output(message: &str) {
  let stdout = stdout();
  let width = message.chars().count();

  let mut writer = BufWriter::new(stdout.lock());
  say(&message, width, &mut writer).unwrap()
}

pub fn format_ferris(message: &str) -> String {
  let width = message.chars().count();

  let buf = Vec::new();
  let mut writer = BufWriter::new(buf);

  let _ = say(message, width, &mut writer);
  let output = writer.into_inner().unwrap();

  String::from_utf8(output).unwrap()
}

pub fn run_command(cmd: &str, args: &[&str]) {
  let full_command = format!("{} {}", cmd, args.join(" "));
  println!("Running command: {}", full_command);

  if cfg!(debug_assertions) {
    return;
  }

  Command::new(cmd)
    .args(args)
    .status()
    .expect(format!("Process failed to execute: {full_command}\nError").as_str());
}

pub fn get_pwa_headers(req: &Request) -> String {
  let context_manager = req.rocket().state::<HandlebarsContextManager>().unwrap();
  let headers: HashMap<String, String> = HashMap::new();
  let rendered_headers = context_manager.render("pwa", headers);

  RawHtml(rendered_headers).0
}
