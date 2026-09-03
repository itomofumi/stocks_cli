use chrono::{DateTime, FixedOffset, TimeZone};
use clap::Parser;
use reqwest::StatusCode;
use serde::Deserialize;
use std::error::Error;
// textplots の Chart は JSON 側の Chart と名前がぶつかるので別名を付ける
use textplots::{Chart as TextChart, Plot, Shape};

/// コマンドライン引数の定義。
/// derive(Parser) を付けると、この struct から clap が自動でパーサを生成する。
/// ドキュメンテーションコメント（///）がそのまま --help の説明文になる。
#[derive(Parser)]
#[command(
    version,
    about = "指定した銘柄の株価を取得して表示する",
    // after_help に書いた内容は -h / --help の末尾にそのまま表示される
    after_help = "\
銘柄コード:
  日本株   4桁の証券コード + .T   7203.T=トヨタ, 6758.T=ソニー, 9432.T=NTT
  米国株   ティッカーをそのまま   AAPL=アップル, MSFT, NVDA
  指数     ^ 始まり               ^N225=日経平均, ^GSPC=S&P500
  ※ 会社名（toyota / トヨタ）では指定できません

期間 (-r, --range):
  1d  5d  1mo  3mo  6mo  ytd  1y  2y  5y

例:
  stocks_cli                      トヨタを直近5営業日分（既定）
  stocks_cli 6758.T               ソニーを表示
  stocks_cli 7203.T 6758.T AAPL   複数銘柄をまとめて表示
  stocks_cli AAPL -r 1mo          アップルを1か月分
  stocks_cli ^N225 --no-chart     日経平均をグラフなしで表示

終了コード:
  0 = 全銘柄の取得に成功   1 = 1銘柄でも失敗（エラーは標準エラー出力へ）"
)]
struct Args {
    /// 銘柄コード。空白区切りで複数指定できる
    #[arg(default_value = "7203.T")]
    symbols: Vec<String>,

    /// 取得期間
    #[arg(short, long, default_value = "5d")]
    range: String,

    /// グラフを表示しない
    #[arg(long)]
    no_chart: bool,
}

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
    // 引数のパース。--help や --version、不正な入力への対応も clap が行う
    let args = Args::parse();

    // Yahoo は User-Agent がないとリクエストを弾くことがあるので付けておく。
    // Client は使い回すと接続を再利用できるので、ループの外で1つだけ作る。
    let client = reqwest::blocking::Client::builder()
        .user_agent("stocks-cli/0.1")
        .build()?;

    let mut has_error = false;

    for (i, symbol) in args.symbols.iter().enumerate() {
        // 2件目以降は区切り線を挟む
        if i > 0 {
            println!("\n{}\n", "─".repeat(48));
        }

        // 1銘柄が失敗しても残りの銘柄は続ける。
        // ? で main を抜けると後続が表示されないため、ここで受け止めて stderr に出す。
        if let Err(e) = report(&client, symbol, &args) {
            eprintln!("エラー ({symbol}): {e}");
            has_error = true;
        }
    }

    // 1件でも失敗したらシェルに異常終了を伝える
    if has_error {
        std::process::exit(1);
    }

    Ok(())
}

/// 1銘柄分を取得して表示する
fn report(
    client: &reqwest::blocking::Client,
    symbol: &str,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval=1d",
        symbol, args.range
    );

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
        .first()
        .ok_or_else(|| format!("銘柄 {symbol} のデータが取得できませんでした"))?;
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

    println!(
        "\n直近{}営業日の終値 (range={}):",
        history.len(),
        args.range
    );
    for (date, price) in &history {
        println!("  {}  {:>8.1}", date.format("%m/%d (%a)"), price);
    }

    // 折れ線グラフ用の点列。x は「何日目か」の連番、y は終値
    let points: Vec<(f32, f32)> = history
        .iter()
        .enumerate()
        .map(|(i, (_, price))| (i as f32, *price as f32))
        .collect();

    if !args.no_chart {
        println!("\n終値の推移 (縦軸: {}, 横軸: 経過日数)", meta.currency);
        TextChart::new(120, 60, 0.0, (points.len() - 1) as f32)
            .lineplot(&Shape::Lines(&points))
            .display();
    }

    Ok(())
}
