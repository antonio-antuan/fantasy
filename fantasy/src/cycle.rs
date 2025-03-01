use std::path::{Path, PathBuf};

use colored::Colorize;
use tera::{Context, Tera};

use tl_parser::types::TLTokenGroup;

use crate::TokenWrap;

/// fantasy config
#[derive(Debug, Clone, TypedBuilder)]
pub struct Config {
    template_path: PathBuf,
    /// tl schema file path
    file_tl: PathBuf,
    output_dir: PathBuf,
}

impl Config {
    pub fn file_tl(&self) -> &PathBuf {
        &self.file_tl
    }
    pub fn template_path(&self) -> &PathBuf {
        &self.template_path
    }
    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }
}

/// cycle
#[derive(Debug, TypedBuilder)]
pub struct Cycle {
    config: Config,
    /// tera template engine
    renderer: Renderer,
    /// token wrap
    tknwrap: TokenWrap,
}

impl Cycle {
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn tknwrap(&self) -> &TokenWrap {
        &self.tknwrap
    }
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }
}

#[derive(Debug, TypedBuilder)]
pub struct Renderer {
    tera: Tera,
}

impl Renderer {
    pub fn render<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        tpl_file: S,
        write_to: P,
        context: &mut Context,
    ) {
        let write_to = write_to.as_ref();
        let tpl_file = tpl_file.as_ref();

        let mut first_write = false;
        if !write_to.exists() {
            let write_dir = write_to.parent();
            if write_dir.is_none() {
                panic!("Cant not get write dir");
            }
            let write_dir = write_dir.unwrap();
            if !write_dir.exists() {
                std::fs::create_dir_all(write_dir).unwrap();
            }
            first_write = true;
        }
        context.insert("first_write", &first_write);
        match self.tera.render(tpl_file.as_ref(), context) {
            Ok(body) => {
                debug!(
                    "USE TEMPLATE [{}] WRITE TO [{}]",
                    tpl_file.blue(),
                    write_to.to_str().map_or("", |v| v).blue()
                );
                toolkit::fs::append(write_to, body).unwrap();
            }
            Err(e) => panic!("Tera template fail: {:?}", e),
        }
    }
}
