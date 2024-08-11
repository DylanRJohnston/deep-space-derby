#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(more_qualified_paths)]

use std::io;

use worker::{console_log, event};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsoleWriter(Vec<u8>);

impl io::Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        console_log!("{}", std::str::from_utf8(buf).unwrap());

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .pretty()
        .with_writer(ConsoleWriter::default)
        .without_time()
        .init();

    tracing::info!("wasm module initialised");
}

#[cfg(target_arch = "wasm32")]
#[event(fetch)]
#[tracing::instrument(skip_all)]
pub async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::response::Response> {
    use tower::Service;

    let game = app::service::durable_object::DurableObjectGameService { env };

    Ok(app::router::into_outer_router(game).call(req).await?)
}

pub fn main() {}
