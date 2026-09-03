//! 株価取得 CLI のエントリポイント。
//!
//! 実際の処理は各モジュールに任せ、ここでは
//! 「引数を読む → 銘柄ごとに取得・表示する → 終了コードを決める」だけを行う。

mod cli;
mod output;
mod stock;
mod yahoo;

use reqwest::blocking::Client;
use std::error::Error;

use cli::Args;

fn main() -> Result<(), Box<dyn Error>> {
    // 引数のパース。--help や --version、不正な入力への対応も clap が行う
    let args = Args::parse_and_validate();

    // Yahoo は User-Agent がないとリクエストを弾くことがあるので付けておく。
    // Client は使い回すと接続を再利用できるので、ループの外で1つだけ作る。
    let client = Client::builder().user_agent("stocks-cli/0.1").build()?;

    let mut has_error = false;

    for (i, symbol) in args.symbols.iter().enumerate() {
        // 2件目以降は区切り線を挟む
        if i > 0 {
            println!("\n{}\n", "─".repeat(48));
        }

        // 1銘柄が失敗しても残りの銘柄は続ける。
        // ? で main を抜けると後続が表示されないため、ここで受け止めて stderr に出す。
        if let Err(e) = report(&client, symbol, &args) {
            eprintln!("エラー ({}): {e}", truncate_for_display(symbol));
            has_error = true;
        }
    }

    // 1件でも失敗したらシェルに異常終了を伝える
    if has_error {
        std::process::exit(1);
    }

    Ok(())
}

/// エラー表示用に銘柄コードを切り詰める。
///
/// 検証エラーのメッセージ自体は入力の中身を含めていないが、
/// ここで銘柄コードを添えているため、長い入力がそのまま端末へ流れてしまう。
fn truncate_for_display(symbol: &str) -> String {
    const MAX_DISPLAY_LEN: usize = 30;

    if symbol.chars().count() <= MAX_DISPLAY_LEN {
        return symbol.to_string();
    }

    let head: String = symbol.chars().take(MAX_DISPLAY_LEN).collect();
    format!("{head}…(以下略)")
}

/// 1銘柄分を取得して表示する
fn report(client: &Client, symbol: &str, args: &Args) -> Result<(), Box<dyn Error>> {
    let stock = yahoo::fetch(client, symbol, args.range.as_str())?;

    output::print_summary(&stock);

    if stock.history.is_empty() {
        println!("\n期間内の終値データがありませんでした");
        return Ok(());
    }

    output::print_history(&stock, args.range.as_str());

    if !args.no_chart {
        output::print_chart(&stock);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::truncate_for_display;

    #[test]
    fn 短い銘柄コードはそのまま表示する() {
        assert_eq!(truncate_for_display("7203.T"), "7203.T");
    }

    #[test]
    fn 長い入力は切り詰めて表示する() {
        let actual = truncate_for_display(&"A".repeat(10_000));
        assert!(
            actual.chars().count() < 50,
            "切り詰められていない: {actual}"
        );
        assert!(actual.ends_with("…(以下略)"));
    }
}
