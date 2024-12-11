use crate::{
    extractors::Game,
    ports::{
        game_service::InternalServerError,
        game_state::{GameDirectory, GameState},
    },
};
use shared::time::Duration;

/*
   wake_up is a semi private endpoint what is used to wake up the game if the game soft locks
   due to missing a wakeup alarm
*/
pub async fn wake_up<G: GameDirectory>(Game(game): Game<G>) -> Result<(), InternalServerError> {
    game.set_alarm(Duration::from_secs(0)).await?;

    Ok(())
}
