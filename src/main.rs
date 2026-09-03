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
use std::thread;
use std::time::Duration;

use cli::Args;
use stock::Stock;

/// 接続確立までの上限。
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// 1リクエスト全体（接続・送信・受信）の上限。
///
/// reqwest の blocking クライアントは既定で30秒だが、
/// CLI としては長すぎるため短く設定する。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 同時に投げるリクエスト数。
///
/// 上限20銘柄を一斉に投げるとレート制限を受けかねないため、
/// この数ずつまとめて取得する。
const MAX_CONCURRENCY: usize = 4;

fn main() -> Result<(), Box<dyn Error>> {
    // 引数のパース。--help や --version、不正な入力への対応も clap が行う
    let args = Args::parse_and_validate();

    // Yahoo は User-Agent がないとリクエストを弾くことがあるので付けておく。
    // Client は使い回すと接続を再利用でき、スレッド間で共有もできる。
    let client = Client::builder()
        .user_agent("stocks-cli/0.1")
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let mut has_error = false;
    let mut printed = 0;

    // MAX_CONCURRENCY 件ずつ並列に取得する。
    // 表示は取得できた順ではなく、引数で指定された順を保つ。
    for chunk in args.symbols.chunks(MAX_CONCURRENCY) {
        for (symbol, result) in chunk.iter().zip(fetch_chunk(&client, chunk, &args)) {
            // 2件目以降は区切り線を挟む
            if printed > 0 {
                println!("\n{}\n", "─".repeat(48));
            }
            printed += 1;

            // 1銘柄が失敗しても残りの銘柄は続ける
            match result {
                Ok(stock) => print_stock(&stock, &args),
                Err(e) => {
                    eprintln!("エラー ({symbol}): {e}");
                    has_error = true;
                }
            }
        }
    }

    // 1件でも失敗したらシェルに異常終了を伝える
    if has_error {
        std::process::exit(1);
    }

    Ok(())
}

/// 複数銘柄を並列に取得する。
///
/// 戻り値は引数と同じ順番に並ぶ（join を順に呼んでいるため）。
///
/// yahoo::fetch のエラーは Box<dyn Error> で Send ではなく、
/// そのままではスレッド境界を越えられない。文字列に変換して返す。
fn fetch_chunk(client: &Client, symbols: &[String], args: &Args) -> Vec<Result<Stock, String>> {
    thread::scope(|scope| {
        let handles: Vec<_> = symbols
            .iter()
            .map(|symbol| {
                // scope 内のスレッドは呼び出し元の変数を借用できる。
                // スコープを抜けるまでに必ず join されるため 'static は要らない。
                scope.spawn(move || {
                    yahoo::fetch(client, symbol, args.range.as_str()).map_err(|e| e.to_string())
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err("取得スレッドが異常終了しました".to_string()))
            })
            .collect()
    })
}

/// 取得済みの1銘柄を表示する
fn print_stock(stock: &Stock, args: &Args) {
    output::print_summary(stock);

    if stock.history.is_empty() {
        println!("\n期間内の終値データがありませんでした");
        return;
    }

    output::print_history(stock, args.range.as_str());

    if !args.no_chart {
        output::print_chart(stock);
    }
}
