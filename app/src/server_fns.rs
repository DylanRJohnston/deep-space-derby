use leptos::prelude::*;
use shared::models::commands::{API, CommandHandler};
use shared::models::game_code::GameCode;
use std::future::Future;

#[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
pub fn server_fn<C: CommandHandler + API>(
    game_id: GameCode,
    input: &C::Input,
) -> impl use<C> + Future<Output = Result<(), ServerFnError>> + Send + 'static {
    use gloo_net::http::Request;
    use worker::send::SendFuture;

    let req = Request::post(&C::url(game_id)).json(input);

    SendFuture::new(async {
        let response = req?.send().await?;
        let status_code = response.status();

        if status_code != 200 {
            let error_body = response.text().await?;

            return Err(ServerFnError::ServerError(format!(
                "expected 200, got {}, body: {}",
                status_code, error_body
            )));
        }

        Ok(())
    })
}

// TODO make this invoke the Durable Object on the server for better SSR, for now, just block forever
#[cfg(feature = "ssr")]
#[allow(unused_variables)]
pub fn server_fn<C: CommandHandler + API>(
    game_id: GameCode,
    input: &C::Input,
) -> impl use<C> + Future<Output = Result<(), ServerFnError>> + Send + 'static {
    async { std::future::pending().await }
}
