use ferris_says::say;
use rocket::Request;
use rocket_dyn_templates::context;
use rocket_include_handlebars::HandlebarsContextManager;
use std::{
  io::{stdout, BufWriter},
  process::{Command, Output},
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

pub fn run_command(cmd: &str, args: &[&str]) -> Option<Output> {
  if cfg!(debug_assertions) {
    let full_command = format!("{} {}", cmd, args.join(" "));
    println!("Running command: {}", full_command);

    return None;
  }

  let output = Command::new(cmd)
    .args(args)
    .output()
    .expect("failed to execute process");

  Some(output)
}

pub fn get_pwa_headers(req: &Request) -> String {
  let context_manager = req.rocket().state::<HandlebarsContextManager>().unwrap();

  context_manager.render("pwa", context! {})
}
