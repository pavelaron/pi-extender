use ferris_says::say;
use rocket::Request;
use rocket_dyn_templates::context;
use rocket_include_handlebars::HandlebarsContextManager;
use std::{
  io::{stdout, Stdout, StdoutLock, BufWriter},
  process::Command,
};

pub fn output(message: &str) {
  let stdout: Stdout = stdout();
  let width: usize = message.chars().count();

  let mut writer: BufWriter<StdoutLock<'static>> = BufWriter::new(stdout.lock());
  say(&message, width, &mut writer).unwrap()
}

pub fn format_ferris(message: &str) -> String {
  let width: usize = message.chars().count();

  let buf: Vec<u8> = Vec::new();
  let mut writer: BufWriter<Vec<u8>> = BufWriter::new(buf);

  let _ = say(message, width, &mut writer);
  let output: Vec<u8> = writer.into_inner().unwrap();

  String::from_utf8(output).unwrap()
}

pub fn run_command(cmd: &str, args: &[&str]) {
  if cfg!(debug_assertions) {
    let full_command: String = format!("{} {}", cmd, args.join(" "));
    println!("Running command: {}", full_command);

    return;
  }

  let _ = Command::new(cmd)
    .args(args)
    .output()
    .expect("failed to execute process");
}

pub fn get_pwa_headers(req: &Request) -> String {
  let context_manager: &HandlebarsContextManager = req.rocket().state::<HandlebarsContextManager>().unwrap();

  context_manager.render("pwa", context! {})
}
