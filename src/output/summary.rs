//! 数値の一覧表示

use crate::stock::Stock;

/// 現在値まわりのサマリー
pub fn print_summary(stock: &Stock) {
    let diff = stock.diff();
    // マイナスの符号は数値側に付くので、プラスのときだけ補う
    let sign = if diff >= 0.0 { "+" } else { "" };
    let diff_percent = stock.diff_percent();

    println!("{} ({})", stock.name, stock.symbol);
    println!("時刻: {}", stock.time.format("%Y-%m-%d %H:%M:%S JST"));
    println!(
        "株価: {:.1} {} ({sign}{diff:.1} / {sign}{diff_percent:.2}%)",
        stock.price, stock.currency
    );
    println!("高値: {:.1} / 安値: {:.1}", stock.day_high, stock.day_low);
    println!("前日終値: {:.1}", stock.previous_close);
    println!("出来高: {}", stock.volume);
}

/// 期間内の終値の一覧
pub fn print_history(stock: &Stock, range: &str) {
    println!(
        "\n直近{}営業日の終値 (range={}):",
        stock.history.len(),
        range
    );
    for day in &stock.history {
        println!("  {}  {:>8.1}", day.date.format("%m/%d (%a)"), day.close);
    }
}
