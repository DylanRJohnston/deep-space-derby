use wasm_bindgen::prelude::*;
use worker::{
    console_log, durable_object, Env, State, WebSocket, WebSocketIncomingMessage, WebSocketPair,
};
use worker_sys::console_error;

use crate::sessions::{Metadata, Session, Sessions};

#[durable_object]
pub struct Example {
    count: usize,
    state: State,
    sessions: Sessions,
}

#[durable_object]
impl DurableObject for Example {
    pub fn new(state: State, _env: Env) -> Self {
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

        Self {
            count: 0,
            state,
            sessions,
        }
    }

    pub async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
        match req.path().as_str() {
            "/connect" => self.on_connect(),
            "/increment" => self.on_increment(),
            _ => worker::Response::error("Not found", 404),
        }
    }

    pub async fn websocket_message(
        &mut self,
        incoming_socket: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        let message = match message {
            WebSocketIncomingMessage::String(message) => Ok(message),
            WebSocketIncomingMessage::Binary(_) => Err("Binary messages not supported"),
        }?;

        for Session { socket, metadata } in self.sessions.iter() {
            if socket == &incoming_socket {
                continue;
            }

            let _ = socket.send_with_str(&format!("{}: {}", metadata.username, message));
        }

        Ok(())
    }

    pub async fn websocket_close(
        &mut self,
        ws: WebSocket,
        code: usize,
        reason: String,
        was_clean: bool,
    ) -> worker::Result<()> {
        if let Some(session) = self.sessions.remove(&ws) {
            for Session { socket, .. } in self.sessions.iter() {
                socket.send_with_str(&format!(
                    "{} disconnected, code: {}, reason: {}, wasClean: {}",
                    session.metadata.username, code, reason, was_clean
                ))?;
            }
        }

        Ok(())
    }

    pub async fn websocket_error(
        &mut self,
        ws: WebSocket,
        error: worker::Error,
    ) -> worker::Result<()> {
        if let Some(session) = self.sessions.remove(&ws) {
            for Session { socket, .. } in self.sessions.iter() {
                socket.send_with_str(&format!(
                    "{} experienced error, {}",
                    session.metadata.username, error
                ))?;
            }
        }

        Ok(())
    }
}

#[wasm_bindgen]
impl Example {
    fn on_connect(&mut self) -> worker::Result<worker::Response> {
        let pair = WebSocketPair::new()
            .map_err(|err| {
                console_error!("Failed to create websocket pair");
                err
            })
            .unwrap();

        let metadata = Metadata {
            username: format!("User {}", self.sessions.len() + 1),
            reloaded_count: 0,
        };

        pair.server.serialize_attachment(&metadata)?;

        self.sessions.insert(Session {
            metadata,
            socket: pair.server.clone(),
        });
        self.state.accept_websocket(pair.server.clone());

        worker::Response::from_websocket(pair.client)
    }

    fn on_increment(&mut self) -> worker::Result<worker::Response> {
        self.count += 1;

        for Session { socket, .. } in self.sessions.iter() {
            socket.send_with_str(format!("Count: {}", self.count))?;
        }

        worker::Response::ok("Increment")
    }
}
