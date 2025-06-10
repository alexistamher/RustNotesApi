use std::{ sync::{ atomic::{ AtomicBool, Ordering }, Arc, LazyLock, Mutex } };

use serde::Serialize;
use tokio::{ task::JoinHandle, time::sleep };

static CACHE: TokenBlacklist = TokenBlacklist {
    tokens: Mutex::new(vec![]),
    current_tokenid: Mutex::new(None),
    should_loop: LazyLock::new(|| Arc::new(AtomicBool::new(true))),
    thread: LazyLock::new(|| Arc::new(Mutex::new(None))),
};

pub fn revoke_token(jti: &str, expiration: i64) {
    if blacklist_is_empty() {
        toggle_loop(true);
    }
    if !is_in_blacklist(jti) {
        push_blacklist(jti, expiration);
    }

    if let Some(current) = get_current_tokenid() {
        if current.jti != jti && current.expiration > expiration {
            kill_current_thread();
        } else {
            return;
        }
    }

    let thread = tokio::spawn(async move {
        let loop_handler = Arc::clone(&CACHE.should_loop);
        while loop_handler.load(Ordering::Relaxed) {
            let next = get_next_tokenid();
            set_current_tokeind(next.clone());
            let waiting_time = next.expiration - chrono::Utc::now().timestamp();
            if waiting_time > 0 {
                sleep(chrono::Duration::seconds(waiting_time).to_std().unwrap()).await;
            }
            pop_from_blacklist(&next.jti);
            if blacklist_is_empty() {
                unset_current();
                toggle_loop(false);
            }
        }
        drop(loop_handler);
    });
    if is_active_thread() {
        kill_current_thread();
    }
    set_current_thread(thread);
}

fn push_blacklist(jti: &str, expiration: i64) {
    let locked_tokens = CACHE.tokens.lock();
    let mut tokens = locked_tokens.unwrap();
    tokens.push(TokenId { jti: jti.to_owned().to_string(), expiration });
    drop(tokens);
}

fn pop_from_blacklist(jti: &str) {
    let locked_tokens = CACHE.tokens.lock();
    let mut tokens = locked_tokens.unwrap();
    if let Some(index) = tokens.iter().position(|t| t.jti == jti) {
        tokens.remove(index);
    }
    drop(tokens);
}

pub fn is_in_blacklist(jti: &str) -> bool {
    let locked_tokens = CACHE.tokens.lock();
    let tokens = locked_tokens.unwrap();
    let exist = tokens.iter().any(|t| t.jti == jti);
    drop(tokens);
    exist
}

fn set_current_tokeind(token_id: TokenId) {
    let locked_current = CACHE.current_tokenid.lock();
    let mut current = locked_current.unwrap();
    *current = Some(token_id);
    drop(current);
}

fn unset_current() {
    let locked_current = CACHE.current_tokenid.lock();
    let mut current = locked_current.unwrap();
    *current = None;
    drop(current);
}

fn get_next_tokenid() -> TokenId {
    let locked_tokens = CACHE.tokens.lock();
    let tokens = locked_tokens.unwrap();
    let exist = tokens
        .iter()
        .min_by(|a, b| a.expiration.cmp(&b.expiration))
        .unwrap()
        .to_owned();
    drop(tokens);
    exist
}

fn get_current_tokenid() -> Option<TokenId> {
    let locked_current = CACHE.current_tokenid.lock();
    let current = locked_current.unwrap();
    let is_current = current.clone();
    drop(current);
    is_current
}

fn blacklist_is_empty() -> bool {
    let locked_tokens = CACHE.tokens.lock();
    let tokens = locked_tokens.unwrap();
    let is_empty = tokens.is_empty();
    drop(tokens);
    is_empty
}

fn toggle_loop(is_on: bool) {
    Arc::new(&CACHE.should_loop).store(is_on, Ordering::Relaxed);
}

fn is_active_thread() -> bool {
    let locked_thread = CACHE.thread.lock();
    let current_thread = locked_thread.unwrap();
    let is_some = current_thread.is_some();
    drop(current_thread);
    is_some
}

fn kill_current_thread() {
    let locked_thread = CACHE.thread.lock();
    let current_thread = locked_thread.unwrap();
    let thread = current_thread.as_ref().unwrap();
    thread.abort();
    drop(current_thread);
}

fn set_current_thread(thread: JoinHandle<()>) {
    let locked_thread = CACHE.thread.lock();
    let mut current_thread = locked_thread.unwrap();
    *current_thread = Some(thread);
}
struct TokenBlacklist {
    tokens: Mutex<Vec<TokenId>>,
    current_tokenid: Mutex<Option<TokenId>>,
    should_loop: LazyLock<Arc<AtomicBool>>,
    thread: LazyLock<Arc<Mutex<Option<JoinHandle<()>>>>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TokenId {
    jti: String,
    expiration: i64,
}
