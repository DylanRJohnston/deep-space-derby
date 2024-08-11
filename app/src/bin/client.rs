use std::io;

use app::screens;
use leptos::{leptos_dom::logging::console_log, mount_to_body};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsoleWriter(Vec<u8>);

impl io::Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        console_log(std::str::from_utf8(buf).unwrap());

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

    mount_to_body(screens::App);
}
