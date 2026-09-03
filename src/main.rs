use chrono::{DateTime, FixedOffset, TimeZone};
use serde::Deserialize;
use std::error::Error;
// textplots の Chart は JSON 側の Chart と名前がぶつかるので別名を付ける
use textplots::{Chart as TextChart, Plot, Shape};

/// トヨタ自動車の東証銘柄コード（.T は東京証券取引所を表す）
const SYMBOL: &str = "7203.T";

/// 取得する期間。5d = 直近5営業日（＝1週間分）
const RANGE: &str = "5d";

// --- ここから JSON の受け皿となる構造体 ---
// Yahoo Finance のレスポンスは
//   { "chart": { "result": [ { "meta": {...}, "timestamp": [...], "indicators": {...} } ] } }
// という入れ子構造。必要な階層だけ struct として定義すれば、他のフィールドは無視される。

#[derive(Deserialize)]
struct ChartResponse {
    chart: Chart,
}

#[derive(Deserialize)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Deserialize)]
struct ChartResult {
    meta: Meta,
    /// 各営業日の時刻（UNIX秒）。indicators の配列と同じ順番で並んでいる
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
}

#[derive(Deserialize)]
struct Quote {
    /// 終値。データ欠損の日は null になるので Option で受ける
    close: Vec<Option<f64>>,
}

// JSON 側は longName のようなキャメルケース、Rust 側はスネークケースが慣習。
// rename_all を付けると serde が自動で対応付けてくれる。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    symbol: String,
    currency: String,
    long_name: String,
    regular_market_price: f64,
    regular_market_time: i64,
    regular_market_day_high: f64,
    regular_market_day_low: f64,
    regular_market_volume: u64,
    chart_previous_close: f64,
}

/// UNIX秒を JST（UTC+9）の日時に変換する
fn to_jst(unixtime: i64) -> Result<DateTime<FixedOffset>, Box<dyn Error>> {
    let jst = FixedOffset::east_opt(9 * 3600).ok_or("タイムゾーンの生成に失敗しました")?;
    let datetime = jst
        .timestamp_opt(unixtime, 0)
        .single()
        .ok_or("時刻の変換に失敗しました")?;
    Ok(datetime)
}

fn main() -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{SYMBOL}?range={RANGE}&interval=1d"
    );

    // Yahoo は User-Agent がないとリクエストを弾くことがあるので付けておく
    let client = reqwest::blocking::Client::builder()
        .user_agent("stocks-cli/0.1")
        .build()?;

    let response = client.get(&url).send()?.error_for_status()?;
    let body: ChartResponse = response.json()?;

    // result は配列。銘柄が見つからない場合は空になるので、その時はエラーにする
    let result = body
        .chart
        .result
        .first()
        .ok_or("銘柄データが取得できませんでした")?;
    let meta = &result.meta;

    // --- 1週間分の終値を取り出す ---

    let quote = result
        .indicators
        .quote
        .first()
        .ok_or("価格データが取得できませんでした")?;

    // 日付と終値をペアにする。終値が null の日（休場など）は除外する
    let mut history: Vec<(DateTime<FixedOffset>, f64)> = Vec::new();
    for (unixtime, close) in result.timestamp.iter().zip(quote.close.iter()) {
        if let Some(price) = close {
            history.push((to_jst(*unixtime)?, *price));
        }
    }

    // --- 現在値のサマリー ---

    let timestamp = to_jst(meta.regular_market_time)?;

    // 前日終値は取得した系列の「最後から2番目」。
    // meta.chart_previous_close は取得期間より前の終値を指すため、ここでは使えない。
    let previous_close = match history.len() {
        0 | 1 => meta.chart_previous_close,
        n => history[n - 2].1,
    };

    // 前日終値との差を計算する
    let diff = meta.regular_market_price - previous_close;
    let diff_percent = diff / previous_close * 100.0;
    let sign = if diff >= 0.0 { "+" } else { "" }; // マイナスの符号は数値側に付くので、プラスのときだけ補う

    println!("{} ({})", meta.long_name, meta.symbol);
    println!("時刻: {}", timestamp.format("%Y-%m-%d %H:%M:%S JST"));
    println!(
        "株価: {:.1} {} ({sign}{diff:.1} / {sign}{diff_percent:.2}%)",
        meta.regular_market_price, meta.currency
    );
    println!(
        "高値: {:.1} / 安値: {:.1}",
        meta.regular_market_day_high, meta.regular_market_day_low
    );
    println!("前日終値: {previous_close:.1}");
    println!("出来高: {}", meta.regular_market_volume);

    // --- 1週間のグラフ ---

    if history.is_empty() {
        println!("\n期間内の終値データがありませんでした");
        return Ok(());
    }

    println!("\n直近{}営業日の終値:", history.len());
    for (date, price) in &history {
        println!("  {}  {:>8.1}", date.format("%m/%d (%a)"), price);
    }

    // 折れ線グラフ用の点列。x は「何日目か」の連番、y は終値
    let points: Vec<(f32, f32)> = history
        .iter()
        .enumerate()
        .map(|(i, (_, price))| (i as f32, *price as f32))
        .collect();

    println!("\n終値の推移 (縦軸: {}, 横軸: 経過日数)", meta.currency);
    TextChart::new(120, 60, 0.0, (points.len() - 1) as f32)
        .lineplot(&Shape::Lines(&points))
        .display();

    Ok(())
}
