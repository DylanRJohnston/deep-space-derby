use worker::{send::SendFuture, Stub};

use crate::ports::game_service::{GameBy, GameRequest, GameService};

pub fn from_env(env: worker::Env) -> impl GameService {
    tower::service_fn(move |GameRequest { by, req }: GameRequest| {
        let stub: worker::Result<Stub> = try {
            match by {
                GameBy::ID(id) => env
                    .durable_object("GAME")?
                    .id_from_string(&id)?
                    .get_stub()?,
                GameBy::Code(code) => env
                    .durable_object("GAME")?
                    .id_from_name(&code.to_string())?
                    .get_stub()?,
            }
        };

        SendFuture::new(async move {
            Ok(stub?
                .fetch_with_request(req.try_into()?)
                .await?
                .try_into()?)
        })
    })
}
