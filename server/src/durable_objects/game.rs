use cookie::Cookie;
use im::Vector;
use serde::{Deserialize, Serialize};

use uuid::Uuid;
use worker::{
    console_error, console_log, durable_object, Env, ListOptions, Request, Result, RouteContext,
    Router, State, Storage, WebSocket, WebSocketPair,
};

use crate::models::{
    commands::{self, Command},
    events::Event,
};

use super::{Metadata, Session, Sessions};

#[durable_object]
pub struct Game {
    state: State,
    events: EventLog,
    env: Env,
    sessions: Sessions,
}

#[durable_object]
impl DurableObject for Game {
    pub fn new(state: State, env: Env) -> Self {
        // Recover listeners from hibernation
        let recovered_sockets = state.get_websockets();

        let mut sessions = Sessions::new();

        for listener in recovered_sockets.into_iter() {
            match listener.deserialize_attachment::<Metadata>() {
                Ok(Some(metadata)) => {
                    sessions.insert(Session {
                        metadata,
                        socket: listener,
                    });
                }
                Ok(None) => console_log!("No metadata found"),
                Err(err) => console_error!("Metadata failed to load: {}", err.to_string()),
            }
        }

        let events = EventLog::new(state.storage());

        Self {
            state,
            events,
            env,
            sessions,
        }
    }

    pub async fn fetch(&mut self, req: Request) -> worker::Result<worker::Response> {
        let env = self.env.clone();

        Router::with_data(self)
            .get_async("/api/object/game/by_code/:code/connect", Self::on_connect)
            .register_command::<commands::CreateGame>()
            .register_command::<commands::JoinGame>()
            .register_command::<commands::ChangeProfile>()
            .register_command::<commands::ReadyPlayer>()
            .run(req, env)
            .await
    }

    pub async fn websocket_close(
        &mut self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        self.sessions.remove(&ws);

        Ok(())
    }

    pub async fn websocket_error(
        &mut self,
        ws: WebSocket,
        _error: worker::Error,
    ) -> worker::Result<()> {
        self.sessions.remove(&ws);

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Counter {
    pub count: usize,
}

impl Game {
    async fn on_connect(
        req: Request,
        ctx: RouteContext<&mut Game>,
    ) -> worker::Result<worker::Response> {
        let game = ctx.data;

        let session_id = match extract_session_id(&req) {
            None => {
                return worker::Response::error("cannot connect to game without session_id", 400)
            }
            Some(session_id) => session_id,
        };

        let pair = WebSocketPair::new()
            .map_err(|err| {
                console_error!("Failed to create websocket pair");
                err
            })
            .unwrap();

        let metadata = Metadata { session_id };
        pair.server.serialize_attachment(&metadata)?;

        game.sessions.insert(Session {
            metadata,
            socket: pair.server.clone(),
        });

        game.state.accept_websocket(&pair.server);

        for event in game.events.iter().await? {
            pair.server.send(event)?;
        }

        worker::Response::from_websocket(pair.client)
    }

    async fn add_event(&mut self, event: Event) -> Result<()> {
        self.events.push(event.clone()).await?;

        self.sessions.broadcast(&event)
    }
}

pub struct EventLog {
    storage: Storage,
    events: Vector<Event>,
    hydrated: bool,
}

impl EventLog {
    fn new(storage: Storage) -> EventLog {
        EventLog {
            storage,
            events: Vector::new(),
            hydrated: false,
        }
    }

    async fn hydrate(&mut self) -> Result<()> {
        if self.hydrated {
            return Ok(());
        }

        let events = self
            .storage
            .list_with_options(ListOptions::new().prefix("EVENT#"))
            .await?;

        events.for_each(&mut |value, _key| {
            let event = serde_wasm_bindgen::from_value::<Event>(value)
                .expect("unable to deserialize value from storage during rehydration");

            self.events.push_back(event);
        });

        self.hydrated = true;

        Ok(())
    }

    async fn push(&mut self, event: Event) -> Result<()> {
        self.hydrate().await?;

        let key = format!("EVENT#{}", self.events.len());

        self.storage.put(&key, &event).await?;
        self.events.push_back(event);

        Ok(())
    }

    async fn iter(&mut self) -> Result<impl Iterator<Item = &Event> + '_> {
        self.hydrate().await?;

        Ok(self.events.iter())
    }

    async fn vector(&mut self) -> Result<&Vector<Event>> {
        self.hydrate().await?;

        Ok(&self.events)
    }
}

trait CommandHandler<'handler> {
    fn register_command<C: Command + 'handler>(self) -> Self;
}

impl<'handler, 'game> CommandHandler<'handler> for Router<'handler, &'game mut Game>
where
    'game: 'handler,
{
    fn register_command<C: Command + 'handler>(self) -> Self {
        self.post_async(
            &C::url(":code"),
            middleware::session(middleware::command::<C, _, _, _>(command_handler::<C>)),
        )
    }
}

fn extract_session_id(req: &Request) -> Option<Uuid> {
    let cookie_header = req.headers().get("cookie").ok()??;

    let cookie = Cookie::split_parse(&cookie_header)
        .filter_map(|it| it.ok())
        .find(|it| it.name() == "session_id")?;

    Uuid::parse_str(cookie.value()).ok()
}

mod middleware {
    use std::convert::identity;

    use cookie::CookieBuilder;
    use futures_util::Future;
    use uuid::Uuid;
    use worker::{Request, Response, Result, RouteContext};

    use crate::models::commands::Command;

    use super::extract_session_id;

    pub fn session<Next, Data, Fut>(
        next: Next,
    ) -> impl Copy + Fn(Request, RouteContext<Data>) -> impl Future<Output = Result<Response>>
    where
        Next: Fn(Uuid, Request, RouteContext<Data>) -> Fut + Copy,
        Fut: Future<Output = Result<Response>>,
    {
        identity(move |req: Request, ctx: RouteContext<Data>| async move {
            let session_id = extract_session_id(&req).unwrap_or_else(Uuid::new_v4);

            let cookie = CookieBuilder::new("session_id", session_id.to_string())
                .path("/")
                .secure(true)
                .http_only(false)
                .same_site(cookie::SameSite::Strict)
                .build();

            Ok(next(session_id, req, ctx).await?.with_headers(
                [("Set-Cookie", cookie.to_string().as_str())]
                    .into_iter()
                    .collect(),
            ))
        })
    }

    pub fn command<C, Next, Data, Fut>(
        next: Next,
    ) -> impl Copy + Fn(Uuid, Request, RouteContext<Data>) -> impl Future<Output = Result<Response>>
    where
        C: Command,
        Next: Fn(Uuid, Data, C::Input) -> Fut + Copy,
        Fut: Future<Output = Result<()>>,
    {
        identity(
            move |session_id: Uuid, mut req: Request, ctx: RouteContext<Data>| async move {
                let input = req.json::<C::Input>().await?;

                match next(session_id, ctx.data, input).await {
                    Ok(_) => Response::empty(),
                    Err(err) => Response::error(err.to_string(), 400),
                }
            },
        )
    }
}

async fn command_handler<C: Command>(
    session_id: Uuid,
    game: &mut Game,
    input: C::Input,
) -> Result<()> {
    let events = game.events.vector().await?;

    if let Some(new_event) = C::handle(session_id, events, input)? {
        game.add_event(new_event).await?;
    };

    Ok(())
}
