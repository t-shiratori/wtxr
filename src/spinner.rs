use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const TICK_MS: u64 = 80;

/// 処理中をアニメーションで表示するスピナー
///
/// ```text
/// ⠹ Adding worktree...
/// ✓ Added worktree for 'feature/foo'   ← success() で完了表示
/// ```
pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// スピナーを開始する
    ///
    /// TTY でない場合（CI・リダイレクト）はアニメーションをスキップして
    /// メッセージをそのまま印字する。
    pub fn new(message: &str) -> Self {
        let stop = Arc::new(AtomicBool::new(false));

        // TTY でなければアニメーションしない
        if !atty() {
            eprintln!("{} {}", "·".dimmed(), message);
            return Self { stop, handle: None };
        }

        let stop_clone = Arc::clone(&stop);
        let msg = message.to_string();

        let handle = thread::spawn(move || {
            let mut i = 0usize;
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                // \r でカーソルを行頭に戻して上書き
                print!("\r{} {}", FRAMES[i % FRAMES.len()].cyan(), msg);
                std::io::stdout().flush().ok();
                i = i.wrapping_add(1);
                thread::sleep(Duration::from_millis(TICK_MS));
            }
        });

        Self { stop, handle: Some(handle) }
    }

    /// バックグラウンドスレッドを停止して行を消去する
    fn stop_animation(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
        // ANSI エスケープ: 行全体を消去してカーソルを行頭へ
        print!("\r\x1b[2K");
        std::io::stdout().flush().ok();
    }

    /// 処理成功を表示して終了する
    pub fn success(mut self, message: &str) {
        self.stop_animation();
        println!("{} {}", "✓".green().bold(), message);
    }

    /// 処理失敗を表示して終了する
    pub fn fail(mut self, message: &str) {
        self.stop_animation();
        eprintln!("{} {}", "✗".red().bold(), message);
    }
}

impl Drop for Spinner {
    /// パニックや早期リターン時でも端末表示を壊さないよう行を消去する
    fn drop(&mut self) {
        if self.handle.is_some() {
            self.stop_animation();
        }
    }
}

/// 標準出力が TTY かどうかを判定する
fn atty() -> bool {
    use std::io::IsTerminal;
    // CI 環境ではアニメーションを無効化
    if std::env::var("CI").is_ok() {
        return false;
    }
    std::io::stdout().is_terminal()
}
