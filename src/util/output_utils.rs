use ferris_says::say;
use rocket_dyn_templates:: handlebars::Handlebars;
use std::{
  io::{stdout, BufWriter},
  process::Command,
};

const PWA_TEMPLATE: &str = include_str!("../.././templates/includes/pwa.hbs");

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

pub fn render_pwa_header() -> String {
  let mut handlebars = Handlebars::new();
  let template_name = "pwa_header";

  // Register the template with a name
  handlebars.register_template_string(template_name, PWA_TEMPLATE)
    .expect("Failed to register template");
  
  // Render the template
  handlebars.render(template_name, &())
    .unwrap_or_else(|e| format!("Template rendering error: {}", e))
}
