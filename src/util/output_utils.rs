use std::{
  io::{stdout, BufWriter},
  process::Command,
};
use ferris_says::say;
use super::structs::ErrorContext;

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

pub fn error_context(code: u16, message: &str) -> ErrorContext {
  ErrorContext {
    status: code,
    message: format_ferris(&message),
  }
}

pub fn run_command(cmd: &str) {
  println!("Running command: {}", cmd);

  if cfg!(debug_assertions) {
    return;
  }

  let mut execute = Command::new(cmd);
  execute
    .status()
    .expect(format!("Process failed to execute: {cmd}\nError").as_str());
}
