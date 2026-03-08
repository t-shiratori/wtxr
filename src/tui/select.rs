use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use crate::domain::worktree::Worktree;

/// ワークツリーを複数選択するアプリケーション状態
struct SelectApp {
    items: Vec<Worktree>,
    /// リストのフォーカス位置
    state: ListState,
    /// 各アイテムの選択状態
    selected: Vec<bool>,
    /// Enter で確定されたか
    confirmed: bool,
}

impl SelectApp {
    fn new(items: Vec<Worktree>) -> Self {
        let len = items.len();
        let mut state = ListState::default();
        if len > 0 {
            state.select(Some(0));
        }
        Self {
            items,
            state,
            selected: vec![false; len],
            confirmed: false,
        }
    }

    fn next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let next = self.state.selected().map_or(0, |i| (i + 1) % len);
        self.state.select(Some(next));
    }

    fn previous(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let prev = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        self.state.select(Some(prev));
    }

    fn toggle(&mut self) {
        if let Some(i) = self.state.selected() {
            self.selected[i] = !self.selected[i];
        }
    }

    fn selected_worktrees(self) -> Vec<Worktree> {
        self.items
            .into_iter()
            .zip(self.selected)
            .filter_map(|(wt, sel)| if sel { Some(wt) } else { None })
            .collect()
    }
}

/// ワークツリーを対話的に複数選択する
///
/// ユーザーが Enter で確定した場合は選択されたワークツリーを返す。
/// q / Esc でキャンセルした場合は `None` を返す。
pub fn select_worktrees(worktrees: Vec<Worktree>) -> anyhow::Result<Option<Vec<Worktree>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = SelectApp::new(worktrees);
    let result = run_app(&mut terminal, &mut app);

    // エラーが起きても必ず端末を元に戻す
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;

    if app.confirmed {
        Ok(Some(app.selected_worktrees()))
    } else {
        Ok(None)
    }
}

/// イベントループ
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut SelectApp,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            // キーダウンイベントのみ処理（Windows での二重発火を防ぐ）
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Char(' ') => app.toggle(),
                KeyCode::Enter => {
                    app.confirmed = true;
                    break;
                }
                KeyCode::Char('q') | KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}

/// 画面描画
fn draw(frame: &mut Frame, app: &mut SelectApp) {
    let area = frame.area();
    let items = build_list_items(&app.items, &app.selected);

    let title = " Select worktrees to remove \
        (↑/↓/k/j: move, Space: toggle, Enter: confirm, q/Esc: cancel) ";

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.state);
    render_footer(frame, area);
}

/// リストアイテムを構築する
fn build_list_items<'a>(items: &'a [Worktree], selected: &'a [bool]) -> Vec<ListItem<'a>> {
    items
        .iter()
        .zip(selected)
        .map(|(wt, &sel)| {
            let checkbox = if sel { "[x]" } else { "[ ]" };
            let branch = wt.branch_display();
            let path = wt.path.to_string_lossy();
            let line = Line::from(format!("{}  {}  ({})", checkbox, path, branch));
            ListItem::new(line)
        })
        .collect()
}

/// フッターにヒントを表示する（ウィンドウが十分に広い場合のみ）
fn render_footer(frame: &mut Frame, area: Rect) {
    if area.height < 4 {
        return;
    }
    // フッター領域はリストの最下行
    let footer_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let hint = ratatui::widgets::Paragraph::new(" [x] = selected, [ ] = unselected")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, footer_area);
}
