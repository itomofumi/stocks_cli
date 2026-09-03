//! 標準出力への表示をまとめたモジュール。
//!
//! 中身は summary.rs と chart.rs に分かれているが、
//! 呼び出し側からは output::print_summary のように使えるよう再公開している。

mod chart;
mod summary;

pub use chart::print_chart;
pub use summary::{print_history, print_summary};
