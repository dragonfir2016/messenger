use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug, Clone)]
pub struct UserDatabase {
    users: HashMap<String, String>,
}

impl UserDatabase {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn register(&mut self, username: &str, password: &str) -> Result<(), &'static str> {
        if self.users.contains_key(username) {
            Err("Користувач з таким логіном вже існує")
        } else {
            self.users.insert(username.to_string(), password.to_string());
            Ok(())
        }
    }

    pub fn login(&self, username: &str, password: &str) -> Result<(), &'static str> {
        if let Some(stored_password) = self.users.get(username) {
            if stored_password == password {
                Ok(())
            } else {
                Err("Неправильний пароль")
            }
        } else {
            Err("Користувача не знайдено")
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

pub async fn handle_register(
    body: AuthRequest,
    db: Arc<Mutex<UserDatabase>>,
) -> Result<impl Reply, Rejection> {
    let mut db = db.lock().await;

    match db.register(body.username.as_str(), body.password.as_str()) {
        Ok(_) => Ok(warp::reply::with_status("Реєстрація успішна", StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(err, StatusCode::BAD_REQUEST)),
    }
}

pub async fn handle_login(
    body: AuthRequest,
    db: Arc<Mutex<UserDatabase>>,
) -> Result<impl Reply, Rejection> {
    let db = db.lock().await;

    match db.login(body.username.as_str(), body.password.as_str()) {
        Ok(_) => Ok(warp::reply::with_status("Логін успішний", StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(err, StatusCode::BAD_REQUEST)),
    }
}
