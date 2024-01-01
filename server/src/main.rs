use std::collections::HashMap;

use models::{
    room::{Room, RoomID},
    user::User,
};
use rocket::{get, launch, post, put, routes, serde::json::Json, tokio::sync::RwLock, State};

mod models;
mod routes;

#[derive(Debug, Default)]
struct Rooms(RwLock<HashMap<RoomID, Room>>);

#[post("/room")]
async fn create_room(rooms: &State<Rooms>) -> Json<RoomID> {
    let mut rooms = rooms.0.write().await;

    let room = Room::new();
    let room_id = room.id;

    rooms.insert(room_id, room);

    Json(room_id)
}

#[get("/room/<id>")]
async fn get_room(rooms: &State<Rooms>, id: RoomID) -> Option<Json<Room>> {
    Some(Json(rooms.0.read().await.get(&id)?.clone()))
}

#[put("/room/<room_id>/join")]
async fn join_room(rooms: &State<Rooms>, room_id: RoomID) -> Option<Json<Room>> {
    let mut rooms = rooms.0.write().await;
    let room = rooms.get_mut(&room_id)?;
    let user = User::new();

    room.users.insert(user.id, user);
    Some(Json(room.clone()))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Rooms::default())
        .mount("/", routes![create_room, get_room, join_room])
}
