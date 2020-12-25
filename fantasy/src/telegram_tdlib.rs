use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use case::CaseExt;
use colored::Colorize;
use tera::Context;

use tl_parser::types::TLTokenGroup;

use crate::Cycle;
use crate::tokenwrap::TokenWrap;

pub struct TGClient<'a> {
  cycle: &'a Cycle,
}


impl<'a> TGClient<'a> {
  pub fn new(cycle: &'a Cycle) -> Self {
    Self { cycle }
  }


  pub fn generate(&self) {
    self.clear_output_dir();
    self.gensrc();
  }

  fn clear_output_dir(&self) {
    let dir = self.cycle.config().output_dir();
    debug!("try clear {}", dir.as_path().display());
    if dir.exists() {
      std::fs::remove_dir_all(dir).unwrap();
    }
    debug!("{} removed", dir.as_path().display());
    debug!("try create {}", dir.as_path().display());
    std::fs::create_dir_all(self.cycle.config().output_dir()).unwrap();
    debug!("{} created", dir.as_path().display());
  }

  fn walk_dir(&self, dir: &PathBuf, context: &mut Context) {
    debug!("start process {}", std::fs::canonicalize(dir).unwrap().display());
    for path in dir.read_dir().unwrap() {
        let path = path.unwrap().path();
        if path.is_dir() {
          self.walk_dir(&path, context);
          continue
        }
        if path.file_name().unwrap().to_str().unwrap().starts_with("td_type") {
          debug!("path {:?} skipped because of td type module", path);
          continue
        }
        let file_path = path.strip_prefix("./template").unwrap().as_os_str().to_str().unwrap();
        let out_file_path = self.cycle.config().output_dir().join(path.strip_prefix("./template/rust-tdlib").unwrap().as_os_str().to_str().unwrap());
        self.cycle.renderer().render(file_path,
                                         out_file_path,
                                         context);
    }
  }

  fn gensrc(&self) {
    let config = self.cycle.config();
    let mut context = self.make_context();
    self.walk_dir(config.template_path(), &mut context);
    self.gen_types();
  }

  fn gen_types(&self) {
    let tknwrap = self.cycle.tknwrap();

    let mut context = Context::new();
    let tokens = tknwrap.tokens();
    for token in tokens {
      if tknwrap.is_skip_type(token.name()) { continue }
      let file_name = tknwrap.which_file(token.name());
      context.insert("token", token);
      self.cycle.renderer().render("rust-tdlib/types/td_type.rs",
                                   self.cycle.config().output_dir().join(&format!("types/{}.rs", file_name)[..]),
                                   &mut context);
    }
  }

  fn make_context(&self) -> Context {
    let tknwrap = self.cycle.tknwrap();

    let mut context = Context::new();

    let tokens = tknwrap.tokens();
    context.insert("tokens", tokens);

    let listener: HashMap<&String, &String> = tknwrap.tdtypefill().listener()
      .iter()
      .filter(|(key, value)| {
        tknwrap.tokens().iter()
          .filter(|&token| token.blood() == Some("Update".to_string()))
          .find(|&token| token.name().to_lowercase() == value.to_lowercase())
          .is_none()
      })
      .collect();
    context.insert("listener", &listener);
    let mut file_obj_map = HashMap::new();
    for token in tokens {
      if tknwrap.is_skip_type(token.name()) { continue }
      let file_name = tknwrap.which_file(token.name());
      let mut vec_of_file_obj = file_obj_map.get(&file_name).map_or(vec![], |v: &Vec<TLTokenGroup>| v.clone());
      vec_of_file_obj.push(token.clone());
      file_obj_map.insert(file_name, vec_of_file_obj);
    }
    context.insert("file_obj_map", &file_obj_map);
    context
  }
}
