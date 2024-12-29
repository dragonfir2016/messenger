use warp::ws::{Message, WebSocket};
use tokio::sync::Mutex;
use futures::{StreamExt, SinkExt};
use std::sync::Arc;

use crate::ChatState;

pub async fn handle_connection(ws: WebSocket, state: Arc<Mutex<ChatState>>) {
    let (ws_sender, mut ws_receiver) = ws.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    let mut rx = {
        let chat_state = state.lock().await;
        chat_state.tx.subscribe()
    };

    {
        let chat_state = state.lock().await;
        let mut sender_lock = ws_sender.lock().await; 
        for old_msg in &chat_state.history {
            if sender_lock.send(Message::text(old_msg.clone())).await.is_err() {
                eprintln!("Failed to send history to client (WebSocket closed)");
                return;
            }
        }
    }

    let ws_sender_clone = ws_sender.clone();
    let task = tokio::spawn(async move {
        while let Ok(new_msg) = rx.recv().await {
            let mut sender_lock = ws_sender_clone.lock().await;
            if sender_lock.send(Message::text(new_msg)).await.is_err() {
                break;
            }
        }
        eprintln!("Broadcast loop ended for one client");
    });

    while let Some(result) = ws_receiver.next().await {
        if let Ok(message) = result {
            if message.is_text() {
                if let Ok(text) = message.to_str() {
                    let mut chat_state = state.lock().await;
                    chat_state.history.push(text.to_string());
                    
                    let _ = chat_state.tx.send(text.to_string());
                }
            }
        } else {
            break;
        }
    }

    task.abort();
    eprintln!("WebSocket connection closed for one client");
}
