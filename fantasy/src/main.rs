#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate typed_builder;

use std::path::{Path, PathBuf};

use tera::Tera;

use cycle::*;
use telegram_tdlib::TGClient;
use tl_parser::parser::parser::TLParser;
use tokenwrap::TokenWrap;

mod cycle;
mod tokenwrap;
mod terafill;
mod types;
mod telegram_tdlib;
mod tdfill;

fn main() {
  simple_logger::init().unwrap();
  log::set_max_level(log::LevelFilter::Debug);


  let project_path = Path::new("./");

  let tdtypefill = tdfill::TDTypeFill::new(project_path.join("schema/td_type_fill.toml")).unwrap();

  let config: Config = Config::builder()
    .template_path(project_path.join("template/telegram-tdlib"))
    .output_dir("/home/anton/Projects/telegram-tdlib/src")
    .file_tl(project_path.join("schema/v1.6.0/td_api.tl"))
    .build();

  let mut tera = Tera::new("template/**/*").expect("Can not create Tera template engine.");
  let tokens = TLParser::new(config.file_tl()).parse().unwrap();
  let tknwrap = TokenWrap::new(tokens, tdtypefill.clone());

  terafill::fill(&mut tera, tknwrap.clone());

  let renderer = Renderer::builder().tera(tera).build();


  let cycle: Cycle = Cycle::builder()
    .config(config)
    .tknwrap(tknwrap)
    .renderer(renderer)
    .build();

  TGClient::new(&cycle).generate();
}
