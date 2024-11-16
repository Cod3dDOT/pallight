pub mod compression;
pub mod constants;
pub mod image;

use log::*;
use std::fs::File;
use std::io::Write;

pub use crate::image::format::Image;
pub use crate::image::{decode, encode};

pub fn init_logging() {
    let target = Box::new(File::create("log.txt").expect("Can't create file"));

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(target))
        .filter(Some("lib_pxc"), LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{}] {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}
