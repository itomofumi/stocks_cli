//! Yahoo Finance からの株価取得。
//!
//! 外に公開するのは fetch() だけで、HTTP のやり取りと JSON の形は
//! このモジュールの中に閉じ込めている。

mod model;

use chrono::{DateTime, FixedOffset, TimeZone};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use std::error::Error;

use crate::stock::{DailyClose, Stock};
use model::ChartResponse;

const ENDPOINT: &str = "https://query1.finance.yahoo.com/v8/finance/chart";

/// 指定した銘柄・期間の株価を取得する
pub fn fetch(client: &Client, symbol: &str, range: &str) -> Result<Stock, Box<dyn Error>> {
    let url = format!("{ENDPOINT}/{symbol}?range={range}&interval=1d");

    let response = client.get(&url).send()?;

    // 存在しない銘柄コードには Yahoo が 404 を返す。
    // error_for_status() に任せると HTTP の生エラーがそのまま出てしまうため、
    // その手前でステータスを見て分かりやすい案内に差し替える。
    if response.status() == StatusCode::NOT_FOUND {
        return Err(format!(
            "銘柄コード {symbol} が見つかりませんでした（例: 7203.T, 6758.T, AAPL）"
        )
        .into());
    }

    let body: ChartResponse = response.error_for_status()?.json()?;

    // result は配列。銘柄が見つからない場合は空になるので、その時はエラーにする
    let result = body
        .chart
        .result
        .into_iter()
        .next()
        .ok_or_else(|| format!("銘柄 {symbol} のデータが取得できませんでした"))?;

    let meta = result.meta;
    let quote = result
        .indicators
        .quote
        .into_iter()
        .next()
        .ok_or("価格データが取得できませんでした")?;

    // 日付と終値をペアにする。終値が null の日（休場など）は除外する
    let mut history: Vec<DailyClose> = Vec::new();
    for (unixtime, close) in result.timestamp.iter().zip(quote.close.iter()) {
        if let Some(close) = close {
            history.push(DailyClose {
                date: to_jst(*unixtime)?,
                close: *close,
            });
        }
    }

    // 前日終値は取得した系列の「最後から2番目」。
    // meta.chart_previous_close は取得期間より前の終値を指すため、ここでは使えない。
    let previous_close = match history.len() {
        0 | 1 => meta.chart_previous_close,
        n => history[n - 2].close,
    };

    Ok(Stock {
        name: meta.long_name,
        symbol: meta.symbol,
        currency: meta.currency,
        time: to_jst(meta.regular_market_time)?,
        price: meta.regular_market_price,
        previous_close,
        day_high: meta.regular_market_day_high,
        day_low: meta.regular_market_day_low,
        volume: meta.regular_market_volume,
        history,
    })
}

/// API が返す UNIX 秒を JST（UTC+9）の日時に変換する
fn to_jst(unixtime: i64) -> Result<DateTime<FixedOffset>, Box<dyn Error>> {
    let jst = FixedOffset::east_opt(9 * 3600).ok_or("タイムゾーンの生成に失敗しました")?;
    let datetime = jst
        .timestamp_opt(unixtime, 0)
        .single()
        .ok_or("時刻の変換に失敗しました")?;
    Ok(datetime)
}
