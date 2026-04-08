use std::sync::atomic::{AtomicBool, Ordering};

use colored::Colorize;

/// グローバルな verbosity フラグ
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// `-v` 時のみ表示する詳細メッセージ
pub fn verbose(msg: &str) {
    if is_verbose() {
        eprintln!("{} {}", "·".dimmed(), msg.dimmed());
    }
}
