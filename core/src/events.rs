pub struct LobbyCreated;

pub struct PlayerJoined;

pub struct PlayerReady;

pub struct GameStarted;

pub enum Events {
    LobbyCreated(LobbyCreated),
    PlayerJoined(PlayerJoined),
    PlayerReady(PlayerReady),
    GameStarted(GameStarted),
}
