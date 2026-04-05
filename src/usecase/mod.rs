pub mod add;
pub mod init;
pub mod list;
pub mod remove;

/// dry-run 時に実行予定の操作一覧
pub struct DryRunPlan {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

impl DryRunPlan {
    pub fn print(&self) {
        println!("{}", self.title);
        println!();
        let col_w = self
            .rows
            .iter()
            .map(|(a, _)| a.len())
            .max()
            .unwrap_or(0)
            .max("Action".len());
        println!("  {:<width$}  Value", "Action", width = col_w);
        println!("  {}", "─".repeat(col_w + 2 + 40));
        for (action, value) in &self.rows {
            println!("  {:<width$}  {}", action, value, width = col_w);
        }
    }
}
