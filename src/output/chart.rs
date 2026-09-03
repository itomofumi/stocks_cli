//! ターミナルへの折れ線グラフ描画

use crate::stock::Stock;
use rgb::RGB8;
// textplots の Chart は JSON 側の Chart と名前がぶつかりうるので別名を付ける
use textplots::{Chart as TextChart, ColorPlot, Shape};

const WIDTH: u32 = 120;
const HEIGHT: u32 = 60;

/// 移動平均線の期間。日本株のチャートで一般的な25日線。
const MA_PERIOD: usize = 25;

/// 終値の線の色（水色）
const PRICE_COLOR: RGB8 = RGB8 {
    r: 0x6c,
    g: 0xc0,
    b: 0xff,
};

/// 移動平均線の色（橙）
const MA_COLOR: RGB8 = RGB8 {
    r: 0xff,
    g: 0xa5,
    b: 0x3d,
};

/// 終値の推移をブレイル文字の折れ線で表示する。
/// データが十分にあれば25日移動平均線も重ねる。
pub fn print_chart(stock: &Stock) {
    // 折れ線グラフ用の点列。x は「何日目か」の連番、y は終値
    let points: Vec<(f32, f32)> = stock
        .history
        .iter()
        .enumerate()
        .map(|(i, day)| (i as f32, day.close as f32))
        .collect();

    if points.is_empty() {
        return;
    }

    // 移動平均は先頭 MA_PERIOD-1 件が None になる。値のある点だけを残す
    let averages: Vec<(f32, f32)> = stock
        .moving_average(MA_PERIOD)
        .into_iter()
        .enumerate()
        .filter_map(|(i, average)| average.map(|average| (i as f32, average as f32)))
        .collect();

    println!("\n終値の推移 (縦軸: {}, 横軸: 経過日数)", stock.currency);

    if averages.is_empty() {
        println!(
            "  終値（水色）　※{}日移動平均線は期間内のデータが{}件のため表示していません（{}件必要）",
            MA_PERIOD,
            points.len(),
            MA_PERIOD
        );
    } else {
        println!("  終値（水色） / {MA_PERIOD}日移動平均（橙）");
    }

    let price_shape = Shape::Lines(&points);
    let average_shape = Shape::Lines(&averages);

    let mut chart = TextChart::new(WIDTH, HEIGHT, 0.0, (points.len() - 1) as f32);
    let chart = chart.linecolorplot(&price_shape, PRICE_COLOR);

    if averages.is_empty() {
        chart.display();
    } else {
        chart.linecolorplot(&average_shape, MA_COLOR).display();
    }
}
