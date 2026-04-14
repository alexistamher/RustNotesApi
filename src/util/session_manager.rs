use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use rust_cipher_lib::models::SessionInfo;

pub type SessionStore = LazyLock<Mutex<HashMap<String, SessionInfo>>>;

pub struct SessionStorage {
    pub sessions: SessionStore,
}

pub static CACHE: SessionStorage = SessionStorage {
    sessions: LazyLock::new(|| Mutex::new(HashMap::new())),
};

pub fn add_session(session_id: &str, shared_secret: SessionInfo) {
    let locked_sessions = CACHE.sessions.lock();
    let mut sessions = locked_sessions.unwrap();
    (*sessions).insert(session_id.to_owned(), shared_secret.try_into().unwrap());
    drop(sessions);
}

pub fn pop_session(session_id: &str) {
    let locked_sessions = CACHE.sessions.lock();
    let mut sessions = locked_sessions.unwrap();
    (*sessions).remove(session_id);
    drop(sessions);
}

pub fn check_session(session_id: &str) -> Option<SessionInfo> {
    let locked_sessions = CACHE.sessions.lock();
    let sessions = locked_sessions.unwrap();
    let result = (*sessions).get(session_id).cloned();
    drop(sessions);
    result
}
