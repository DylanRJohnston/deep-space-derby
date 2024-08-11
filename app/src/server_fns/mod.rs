use leptos::ServerFnError;
use shared::models::commands::{CommandHandler, API};
use shared::models::game_id::GameID;
use std::future::Future;

// #[cfg(not(feature = "ssr"))]
pub fn server_fn<C: CommandHandler + API>(
    game_id: GameID,
    input: &C::Input,
) -> impl Future<Output = Result<(), ServerFnError>> {
    use gloo_net::http::Request;

    let req = Request::post(&C::url(game_id)).json(input);

    async {
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
    }
}

// // TODO make this invoke the Durable Object on the server for better SSR, for now, just block forever
// #[cfg(feature = "ssr")]
// #[allow(unused_variables)]
// pub fn server_fn<C: Command>(
//     game_id: GameID,
//     input: &C::Input,
// ) -> impl Future<Output = Result<(), ServerFnError>> + 'static {

//     async { std::future::pending().await }
// }
