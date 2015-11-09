#![allow(dead_code)]

extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;

mod filter;
mod util;

pub mod project;
pub mod manager;
pub mod templater;
pub mod keyword_replacement;

pub use manager::Luigi;
pub use keyword_replacement::IsKeyword;

