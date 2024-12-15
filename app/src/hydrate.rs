use std::io;

use crate::app;
use leptos::{logging::log, prelude::*};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsoleWriter(Vec<u8>);

impl io::Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        log!("{}", std::str::from_utf8(buf).unwrap());

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn main() {}

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(ConsoleWriter::default)
        .without_time()
        .init();

    hydrate_body(app::App);
}
