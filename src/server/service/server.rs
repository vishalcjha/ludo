use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{
    server::entity::action::Response as CommandResponse,
    server::entity::{action::Command, color::Color},
};

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
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            println!("Client disconnected while server tried receiving");
            return;
        };

        let msg = match handle_command(msg) {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("Failure is {:?}", err);
                Message::Text("Failed to handler request".to_owned())
            }
        };

        if socket.send(msg).await.is_err() {
            println!("Client disconnected while server tried sending");
        }
    }
}

fn handle_command(msg: Message) -> Result<Message> {
    let msg = msg.into_text()?;
    let _command = serde_json::from_str::<Command>(&msg)?;

    let response = CommandResponse::make_available_colors(vec![
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
    ]);

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
        let local_addr = app.local_addr();
        tokio::spawn(app);

        println!("Local addr is {}", local_addr);
        let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{local_addr}/game"))
            .await
            .unwrap();

        let request = serde_json::to_string(&Command::AvailableColors).unwrap();
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
            CommandResponse::make_available_colors(vec![
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue
            ])
        );
    }
}
