use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{server::entity::action::Command, server::entity::action::Response as CommandResponse};

use super::state::AppState;
use anyhow::Result;

#[tokio::main]
async fn main() {
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap()).serve(app().into_make_service());
}

fn app() -> Router {
    let app = Router::new()
        .route("/game", get(handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(AppState::new());

    app
}

async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let (sender, receiver) = socket.split();
    let (tx, rx) = channel::<Message>(1);

    tokio::spawn(handle_read(receiver, state.clone(), tx));

    tokio::spawn(handle_write(sender, rx));
}

async fn handle_read(
    mut receiver: SplitStream<WebSocket>,
    state: AppState,
    socket_write_requester: Sender<Message>,
) {
    while let Some(msg) = receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            eprintln!("Client disconnected while server tried receiving");
            return;
        };

        let msg = match handle_command(msg, state.clone()) {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("Failure is {:?}", err);
                let failure_message = CommandResponse::FailureMessage {
                    message: "Failed to handler request".to_owned(),
                }
                .to_json();
                Message::Text(failure_message)
            }
        };

        let _ = socket_write_requester.send(msg).await;
    }
}

async fn handle_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut socket_write_receiver: Receiver<Message>,
) {
    while let Some(msg) = socket_write_receiver.recv().await {
        if sender.send(msg).await.is_err() {
            eprintln!("Client disconnected while server tried sending");
        }
    }
}

fn handle_command(msg: Message, state: AppState) -> Result<Message> {
    let msg = msg.into_text()?;
    let command = serde_json::from_str::<Command>(&msg)?;
    let response = match command {
        Command::CreateGame => {
            let game_id = state.create_game()?;
            CommandResponse::CreateGameResponse { game_id }
        }
        Command::AvailableColors { .. } => todo!(),
        Command::JoinGame { .. } => todo!(),
        Command::StartGame { .. } => todo!(),
    };

    let stringified = serde_json::to_string(&response)?;
    Ok(Message::Text(stringified))
}

#[cfg(test)]
mod test {
    use std::net::{Ipv4Addr, SocketAddr};

    use futures::{SinkExt, StreamExt};

    use crate::{
        server::entity::action::Command, server::entity::action::Response as CommandResponse,
    };

    use super::*;

    #[tokio::test]
    async fn integration_test() {
        let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
        let app = axum::Server::bind(&socket_addr).serve(app().into_make_service());
        let local_address = app.local_addr();
        tokio::spawn(app);

        let (mut socket, _) =
            tokio_tungstenite::connect_async(format!("ws://{local_address}/game"))
                .await
                .unwrap();

        for i in 1..10 {
            let request = serde_json::to_string(&Command::CreateGame).unwrap();
            socket
                .send(tungstenite::Message::text(request))
                .await
                .unwrap();

            let msg = match socket.next().await.unwrap().unwrap() {
                tungstenite::Message::Text(msg) => msg,
                other => panic!("expected text message but got {:#?}", other),
            };

            assert_eq!(
                serde_json::from_str::<CommandResponse>(&msg).unwrap(),
                CommandResponse::CreateGameResponse { game_id: i }
            );
        }
    }
}
