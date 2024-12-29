use warp::Filter;
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;

mod ws_handler;
mod auth;

#[derive(Debug)]
pub struct ChatState {
    pub tx: broadcast::Sender<String>,
    pub history: Vec<String>,
}

#[tokio::main]
async fn main() {
    let chat_state = Arc::new(Mutex::new(ChatState {
        tx: broadcast::channel(100).0,
        history: Vec::new(),
    }));

    let user_db = Arc::new(Mutex::new(auth::UserDatabase::new()));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["content-type", "authorization"]);

    let register_route = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(user_db.clone()))
        .and_then(auth::handle_register)
        .with(cors.clone());

    let login_route = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(user_db.clone()))
        .and_then(auth::handle_login)
        .with(cors.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map({
            let chat_state = chat_state.clone();
            move |ws: warp::ws::Ws| {
                let state_clone = chat_state.clone();
                ws.on_upgrade(move |websocket| ws_handler::handle_connection(websocket, state_clone))
            }
        })
        .with(cors);

    let routes = register_route.or(login_route).or(ws_route);

    println!("Server running on http://127.0.0.1:8080");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_db(
    db: Arc<Mutex<auth::UserDatabase>>,
) -> impl Filter<Extract = (Arc<Mutex<auth::UserDatabase>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || db.clone())
}
