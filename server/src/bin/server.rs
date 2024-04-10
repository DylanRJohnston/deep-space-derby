#![feature(try_blocks)]
#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#![feature(more_qualified_paths)]

use axum::body::Body;
use axum::extract::Request;
use axum::extract::{FromRequestParts, Path};
use axum::http::header::LOCATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{AppendHeaders, Response};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{any, get};
use axum::{async_trait, debug_handler, Extension, Json};
use axum::{
    extract::{Form, State},
    routing::post,
};
use axum_extra::extract::CookieJar;
use client::screens;
use cookie::CookieBuilder;
use im::Vector;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use serde::{Deserialize, Serialize};
use shared::models::commands::{self, JoinGame};
use shared::models::game_id::{self, GameID};
use shared::models::{commands::Command, game_id::generate_game_code};
use shared::models::{commands::Effect, events::Event};
use std::any::type_name;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use tower::Service;
use uuid::Uuid;
use worker::{
    console_error, console_log, console_warn, durable_object, event, Env, ListOptions, Storage,
    WebSocket, WebSocketPair,
};

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

pub enum ErrWrapper {
    Worker(worker::Error),
    Axum(axum::http::Error),
    Json(serde_json::Error),
    Raw(String),
}

impl From<worker::Error> for ErrWrapper {
    fn from(value: worker::Error) -> Self {
        ErrWrapper::Worker(value)
    }
}

impl From<axum::http::Error> for ErrWrapper {
    fn from(value: axum::http::Error) -> Self {
        ErrWrapper::Axum(value)
    }
}

impl From<serde_json::Error> for ErrWrapper {
    fn from(value: serde_json::Error) -> Self {
        ErrWrapper::Json(value)
    }
}

impl From<String> for ErrWrapper {
    fn from(value: String) -> Self {
        ErrWrapper::Raw(value)
    }
}

impl From<Infallible> for ErrWrapper {
    fn from(_: Infallible) -> Self {
        unimplemented!()
    }
}

impl IntoResponse for ErrWrapper {
    fn into_response(self) -> axum::response::Response {
        match self {
            ErrWrapper::Worker(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Axum(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Json(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Raw(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}

#[axum::debug_handler]
#[worker::send]
async fn create_game(
    State(env): State<Env>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, ErrWrapper> {
    let game_code = generate_game_code();

    console_log!("Inside create_game {}", game_code);

    let mut req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        commands::CreateGame::url(game_code)
    ));

    *req.headers_mut().unwrap() = headers;
    req.headers_mut()
        .unwrap()
        .insert("Content-Type", "application/json".parse().unwrap());

    let req = req.body(serde_json::to_string(
        &<commands::CreateGame as Command>::Input { code: game_code },
    )?)?;

    console_log!("Sending create_game to durable object");

    let response = env
        .durable_object("GAME")?
        .id_from_name(game_code.deref())?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?;

    if response.status_code() != 200 {
        return Ok(response.into());
    }

    Ok(Redirect::to(&commands::CreateGame::redirect(game_code).unwrap()).into_response())
}

#[axum::debug_handler]
#[worker::send]
async fn join_game(
    State(env): State<Env>,
    headers: HeaderMap,
    Form(join_game): Form<<JoinGame as Command>::Input>,
) -> Result<Response, ErrWrapper> {
    let mut req = Request::post(format!(
        "https://DURABLE_OBJECT{}",
        JoinGame::url(join_game.code)
    ));

    *req.headers_mut().unwrap() = headers;
    req = req.header("Content-Type", "application/json");

    let req = req.body(serde_json::to_string(&join_game)?)?;

    env.durable_object("GAME")?
        .id_from_name(join_game.code.deref())?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?;

    Ok(Redirect::to(&commands::JoinGame::redirect(join_game.code).unwrap()).into_response())
}

#[axum::debug_handler]
#[worker::send]
async fn forward_command(
    State(env): State<Env>,
    Path((code, _)): Path<(String, String)>,
    req: Request,
) -> Result<Response, ErrWrapper> {
    console_log!("Forwarding command to durable object");

    Ok(env
        .durable_object("GAME")?
        .id_from_name(&code)?
        .get_stub()?
        .fetch_with_request(req.try_into()?)
        .await?
        .into())
}

pub async fn session_middleware(
    session_id: Option<SessionID>,
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> (CookieJar, Response) {
    console_log!("Inside session middleware");

    let mut cookie_jar = cookie_jar;

    let session_id = match session_id {
        Some(session_id) => {
            console_log!("Found session_id, {:?}", session_id);
            session_id
        }
        None => {
            let session_id = SessionID(Uuid::new_v4());
            console_log!("Creating new session_id {:?}", session_id);
            cookie_jar = cookie_jar.add(
                CookieBuilder::new("session_id", session_id.0.to_string())
                    .path("/")
                    .secure(true)
                    .http_only(false)
                    .same_site(cookie::SameSite::Strict)
                    .build(),
            );

            session_id
        }
    };

    request.extensions_mut().insert(session_id);

    (cookie_jar, next.run(request).await)
}

#[event(fetch)]
pub async fn fetch(
    req: worker::HttpRequest,
    env: Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<Body>> {
    let leptos_options = LeptosOptions::builder()
        .output_name("index")
        .site_pkg_dir("pkg")
        .env(leptos_config::Env::DEV)
        .site_addr(SocketAddr::from_str("127.0.0.1:8788").unwrap())
        .reload_port(3001)
        .build();

    let mut router = axum::Router::new()
        .route("/api/create_game", post(create_game))
        .route("/api/join_game", post(join_game))
        .route(
            "/api/object/game/by_code/:code/*command",
            any(forward_command),
        )
        .with_state(env)
        .layer(axum::middleware::from_fn(session_middleware))
        .leptos_routes(
            &leptos_options,
            generate_route_list(screens::App),
            screens::App,
        )
        .with_state(leptos_options);

    let response = router.call(req).await?;

    console_log!("Responding with {:?}", &response);

    Ok(response)
}

pub fn main() {}

#[durable_object]
pub struct Game {
    state: worker::State,
    events: EventLog,
    env: Env,
    // sessions: Sessions,
}

#[durable_object]
impl DurableObject for Game {
    pub fn new(state: State, env: Env) -> Self {
        // Recover listeners from hibernation
        let recovered_sockets = state.get_websockets();

        // let mut sessions = Sessions::new();

        for listener in recovered_sockets.into_iter() {
            match listener.deserialize_attachment::<Metadata>() {
                Ok(Some(metadata)) => {
                    // sessions.insert(Session {
                    //     metadata,
                    //     socket: listener,
                    // });
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
            // sessions,
        }
    }

    pub async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
        console_log!("Inside fetch in durable object");

        console_log!("{:?}", req.headers());

        axum::Router::new()
            .route("/api/object/game/by_code/:code/connect", get(on_connect))
            .register_command::<commands::CreateGame>()
            .register_command::<commands::JoinGame>()
            .register_command::<commands::ChangeProfile>()
            .register_command::<commands::ReadyPlayer>()
            .register_command::<commands::PlaceBets>()
            .with_state(GameWrapper::new(self))
            .call(req.try_into()?)
            .await?
            .try_into()
    }

    pub async fn websocket_close(
        &mut self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        // self.sessions.remove(&ws);

        Ok(())
    }

    pub async fn websocket_error(
        &mut self,
        ws: WebSocket,
        _error: worker::Error,
    ) -> worker::Result<()> {
        // self.sessions.remove(&ws);

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Counter {
    pub count: usize,
}

// Workers are single threaded, but axum has annoying Send + Send + 'static bounds on State
pub struct GameWrapper(*mut Game);

unsafe impl Sync for GameWrapper {}
unsafe impl Send for GameWrapper {}

impl GameWrapper {
    fn new(game: &mut Game) -> Self {
        Self(game as *mut Game)
    }
}

impl Clone for GameWrapper {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Deref for GameWrapper {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for GameWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

#[axum::debug_handler]
#[worker::send]
async fn on_connect(
    State(mut game): State<GameWrapper>,
    SessionID(session_id): SessionID,
) -> Result<Response, ErrWrapper> {
    console_log!("Inside connect handler");

    let pair = WebSocketPair::new()
        .map_err(|err| {
            console_error!("Failed to create websocket pair");
            err
        })
        .unwrap();

    let metadata = Metadata { session_id };
    pair.server.serialize_attachment(&metadata)?;

    // game.sessions.insert(Session {
    //     metadata,
    //     socket: pair.server.clone(),
    // });

    game.state.accept_web_socket(&pair.server);

    console_log!("Sending {:#?}", game.events.vector().await);

    for event in game.events.iter().await? {
        pair.server.send(event)?;
    }

    // worker::Response::from_websocket(pair.client)

    Ok(Response::builder()
        .status(101)
        .extension(pair.client)
        .body(Body::empty())?)
}

impl Game {
    async fn add_event(&mut self, event: Event) -> worker::Result<()> {
        self.events.push(event.clone()).await?;

        for ws in &self.state.get_websockets() {
            ws.send(&event)?;
        }

        Ok(())

        // self.sessions.broadcast(&event)
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

    async fn hydrate(&mut self) -> worker::Result<()> {
        if self.hydrated {
            return Ok(());
        }

        let events = self
            .storage
            .list_with_options(ListOptions::new().prefix("EVENT#"))
            .await?;

        events.for_each(&mut |value, key| {
            let event = serde_wasm_bindgen::from_value::<Event>(value)
                .expect("unable to deserialize value from storage during rehydration");

            console_log!("Rehydration {:#?} {:#?}", key.as_string(), event);

            self.events.push_back(event);
        });

        self.hydrated = true;

        Ok(())
    }

    async fn push(&mut self, event: Event) -> worker::Result<()> {
        self.hydrate().await?;

        let key = format!("EVENT#{:0>5}", self.events.len());

        console_log!("Saving event with key {:#?}", key);

        self.storage.put(&key, &event).await?;
        self.events.push_back(event);

        Ok(())
    }

    async fn iter(&mut self) -> worker::Result<impl Iterator<Item = &Event> + '_> {
        self.hydrate().await?;

        Ok(self.events.iter())
    }

    async fn vector(&mut self) -> worker::Result<&Vector<Event>> {
        self.hydrate().await?;

        Ok(&self.events)
    }
}

trait CommandHandler {
    fn register_command<C: Command + 'static>(self) -> Self;
}

#[worker::send]
pub async fn command_handler<C: Command>(
    SessionID(session_id): SessionID,
    State(mut game): State<GameWrapper>,
    Json(input): Json<C::Input>,
) -> Result<(), ErrWrapper> {
    console_log!("Inside command handler {:?}, {:?}", session_id, input);

    let (new_events, effect) = C::handle(session_id, game.events.vector().await?, input)?;

    for event in new_events {
        game.add_event(event).await?;
    }

    match effect {
        Some(Effect::Alarm(time)) => {
            match game.state.storage().get_alarm().await? {
                Some(_) => {
                    console_warn!(
                        "{} attempted to set an alarm while one was already set, noop",
                        type_name::<C>()
                    )
                }
                None => game.state.storage().set_alarm(time).await?,
            };
        }
        Some(Effect::SoftCommand(f)) => {
            if let Some(event) = f(game.events.vector().await?) {
                game.add_event(event).await?
            }
        }
        None => {}
    }

    Ok(())
}

impl CommandHandler for axum::Router<GameWrapper> {
    fn register_command<C: Command + 'static>(self) -> Self {
        self.route(&C::url(":code"), post(command_handler::<C>))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SessionID(Uuid);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SessionID {
    type Rejection = &'static str;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        console_log!("Inside session_id extractor");

        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        jar.get("session_id")
            .ok_or("missing session_id cookie")
            .map(|it| it.value())
            .and_then(|it| Uuid::parse_str(it).map_err(|_| "Unable to parse session_id"))
            .map(SessionID)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub session_id: Uuid,
}

#[derive(Debug)]
pub struct Session {
    pub metadata: Metadata,
    pub socket: WebSocket,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.socket == other.socket
    }
}

// #[derive(Debug)]
// pub struct Sessions(Vec<Session>);

// impl Sessions {
//     pub fn new() -> Self {
//         Self(Vec::new())
//     }

//     pub fn insert(&mut self, session: Session) {
//         self.0.push(session)
//     }

//     pub fn remove(&mut self, ws: &WebSocket) -> Option<Session> {
//         if let Some(position) = self.0.iter().position(|it| &it.socket == ws) {
//             return Some(self.0.remove(position));
//         }

//         None
//     }

//     pub fn iter(&self) -> impl Iterator<Item = &Session> {
//         self.0.iter()
//     }

//     pub fn broadcast(&self, data: &Event) -> worker::Result<()> {
//         for session in self.iter() {
//             session.socket.send(data)?;
//         }

//         Ok(())
//     }
// }

// impl Default for Sessions {
//     fn default() -> Self {
//         Self::new()
//     }
// }
