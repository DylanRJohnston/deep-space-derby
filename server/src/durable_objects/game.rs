use im::Vector;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use worker::{
    console_error, console_log, durable_object, Env, ListOptions, Request, Response, Result,
    RouteContext, Router, State, Storage, WebSocket, WebSocketPair,
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

        console_log!(
            "Reloaded durable object, {} listeners recovered",
            recovered_sockets.len()
        );

        let mut sessions = Sessions::new();

        for listener in recovered_sockets.into_iter() {
            match listener.deserialize_attachment::<Metadata>() {
                Ok(Some(mut metadata)) => {
                    console_log!("Found metadata {:?}", metadata);

                    metadata.reloaded_count += 1;

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

#[wasm_bindgen]
impl Game {
    async fn on_connect(
        _: Request,
        ctx: RouteContext<&mut Game>,
    ) -> worker::Result<worker::Response> {
        let pair = WebSocketPair::new()
            .map_err(|err| {
                console_error!("Failed to create websocket pair");
                err
            })
            .unwrap();

        let metadata = Metadata {
            username: format!("User {}", ctx.data.sessions.len() + 1),
            reloaded_count: 0,
        };

        pair.server.serialize_attachment(&metadata)?;

        ctx.data.sessions.insert(Session {
            metadata,
            socket: pair.server.clone(),
        });

        ctx.data.state.accept_websocket(&pair.server);

        for event in ctx.data.events.iter().await? {
            pair.server.send(event)?;
        }

        worker::Response::from_websocket(pair.client)
    }

    async fn add_event(&mut self, event: Event) -> Result<()> {
        console_log!("Storing and broadcasting new event, {:?}", event);

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

            console_log!("Rehydrated event {:?}", event);

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

trait CommandHandler {
    fn register_command<C: Command>(self) -> Self;
}

impl<'handler, 'game> CommandHandler for Router<'handler, &'game mut Game>
where
    'game: 'handler,
{
    fn register_command<C: Command>(self) -> Self {
        self.post_async(&C::url(":code"), |mut req, ctx| async move {
            let response: Result<()> = try {
                let input = req.json::<C::Input>().await?;
                let events = ctx.data.events.vector().await?;

                C::precondition(events, &input)?;

                let new_event = C::handle(events, input);
                ctx.data.add_event(new_event).await?;
            };

            match response {
                Ok(_) => Response::empty(),
                Err(err) => Response::error(err.to_string(), 400),
            }
        })
    }
}
