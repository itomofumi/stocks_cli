//! ターミナルへの折れ線グラフ描画

use crate::stock::Stock;
// textplots の Chart は JSON 側の Chart と名前がぶつかりうるので別名を付ける
use textplots::{Chart as TextChart, Plot, Shape};

const WIDTH: u32 = 120;
const HEIGHT: u32 = 60;

/// 終値の推移をブレイル文字の折れ線で表示する
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

    println!("\n終値の推移 (縦軸: {}, 横軸: 経過日数)", stock.currency);
    TextChart::new(WIDTH, HEIGHT, 0.0, (points.len() - 1) as f32)
        .lineplot(&Shape::Lines(&points))
        .display();
}
