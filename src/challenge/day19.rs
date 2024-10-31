use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
    routing::{get, post},
    Router,
};
use futures_util::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{
    broadcast::{self, Sender},
    Mutex,
};

#[derive(Clone)]
pub struct ChatState {
    rooms: Arc<Mutex<HashMap<u32, RoomState>>>,
    total_tweets: Arc<Mutex<u32>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            total_tweets: Arc::new(Mutex::new(0)),
        }
    }
}

pub struct RoomState {
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<TweetMsg>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TweetMsg {
    user: String,
    message: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/ws/ping", get(serve))
        .route("/reset", post(reset))
        .route("/views", get(views))
        .route("/ws/room/:room_number/user/:username", get(serve_chat))
        .with_state(Arc::new(ChatState::new()))
}

pub async fn serve(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(ping)
}

async fn ping(mut socket: WebSocket) {
    let mut started = false;
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            break;
        };
        let msg = msg.to_text().unwrap();
        if msg == "serve" {
            started = true;
        }
        if !started {
            continue;
        }
        if msg == "ping" {
            socket
                .send(Message::from("pong"))
                .await
                .expect("Failed to send message to client");
        }
    }
}

pub async fn reset(State(state): State<Arc<ChatState>>) {
    let mut guard = state.total_tweets.lock().await;
    *guard = 0;
    drop(guard);
    (*state.rooms.lock().await).clear();
}

pub async fn views(State(state): State<Arc<ChatState>>) -> String {
    state.total_tweets.lock().await.to_string()
}

pub async fn serve_chat(
    ws: WebSocketUpgrade,
    Path((room_number, username)): Path<(u32, String)>,
    State(state): State<Arc<ChatState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, room_number, username, state))
}

pub async fn handle_socket(
    socket: WebSocket,
    room_number: u32,
    username: String,
    state: Arc<ChatState>,
) {
    let (sender, receiver) = socket.split();

    // join room
    let Some(tx) = join_room(room_number, username.clone(), state.rooms.clone()).await else {
        // let _ = sender.send(Message::from("Error joining room")).await;
        println!("{} failed to join room {}", username, room_number);
        return;
    };

    println!("{} joined room {}", username, room_number);

    // receive broadcasted messages and send to socket and increment total tweets
    let handle1 = tokio::spawn(broadcast_chat_message(
        sender,
        username.clone(),
        tx.clone(),
        state.total_tweets.clone(),
    ));
    // receive socket and send broadcast messages
    let handle2 = tokio::spawn(send_chat_messages(receiver, tx.clone(), username.clone()));
    // tokio::select! { _ = (&mut handle1) => handle2.abort(), _ = (&mut handle2) => handle1.abort() };
    let _ = tokio::join!(handle1, handle2);

    leave_room(room_number, username, state.rooms.clone()).await;
}

pub async fn join_room(
    room_number: u32,
    username: String,
    rooms: Arc<Mutex<HashMap<u32, RoomState>>>,
) -> Option<Sender<TweetMsg>> {
    let (tx, _) = broadcast::channel(100000);
    let mut rooms = rooms.lock().await;
    let inserted = rooms
        .entry(room_number)
        .or_insert_with(|| RoomState {
            users: Mutex::new(HashSet::new()),
            tx,
        })
        .users
        .lock()
        .await
        .insert(username);

    if inserted {
        Some(rooms.get(&room_number).unwrap().tx.clone())
    } else {
        None
    }
}

pub async fn leave_room(
    room_number: u32,
    username: String,
    rooms: Arc<Mutex<HashMap<u32, RoomState>>>,
) {
    let mut no_room = false;
    // if let Some(room_state) = rooms.lock().await.get(&room_number) {
    let mut rooms = rooms.lock().await;
    if let Some(room_state) = rooms.get(&room_number) {
        let mut users = room_state.users.lock().await;
        users.remove(&username);
        if users.len() == 0 {
            no_room = true;
        }
    }
    if no_room {
        rooms.remove(&room_number);
    }
    println!("{} left room {}", username, room_number);
}

pub async fn broadcast_chat_message(
    mut sender: SplitSink<WebSocket, Message>,
    username: String,
    tx: Sender<TweetMsg>,
    total_tweets: Arc<Mutex<u32>>,
) {
    let mut rx = tx.subscribe();

    // while let Ok(msg) = rx.recv().await {
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if let Err(e) = sender
                    .send(Message::from(serde_json::to_string(&msg).unwrap()))
                    .await
                {
                    println!("[To {}] Error broadcasting chat message: {}", username, e);
                    break;
                }
                *total_tweets.lock().await += 1;
            }
            Err(e) => {
                println!("[To {}] Error rx recv(): {}", username, e);
                break;
            }
        }
    }
}

pub async fn send_chat_messages(
    mut receiver: SplitStream<WebSocket>,
    tx: Sender<TweetMsg>,
    username: String,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        let msg = msg.to_text().unwrap();

        let Ok(msg) = serde_json::from_str::<ChatMsg>(msg) else {
            continue;
        };
        let msg = msg.message;
        if msg.len() > 128 {
            continue;
        }

        let msg = TweetMsg {
            user: username.clone(),
            message: msg,
        };
        if let Err(e) = tx.send(msg) {
            println!("[From {}] Error sending chat message: {}", username, e);
            break;
        }
    }
}
