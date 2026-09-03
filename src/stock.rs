//! アプリ内部で扱う株価データの型。
//!
//! Yahoo のレスポンス形式（yahoo::model）とは意図的に分けている。
//! API の仕様が変わっても、変更範囲を yahoo モジュール内に閉じ込められるため。

use chrono::{DateTime, FixedOffset};

/// ある1営業日の終値
pub struct DailyClose {
    pub date: DateTime<FixedOffset>,
    pub close: f64,
}

/// 1銘柄分の株価情報
pub struct Stock {
    pub name: String,
    pub symbol: String,
    pub currency: String,
    /// 現在値（場が閉じていれば終値）の時刻
    pub time: DateTime<FixedOffset>,
    pub price: f64,
    pub previous_close: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub volume: u64,
    /// 取得期間内の終値。古い順に並ぶ
    pub history: Vec<DailyClose>,
}

impl Stock {
    /// 前日終値からの変化額
    pub fn diff(&self) -> f64 {
        self.price - self.previous_close
    }

    /// 前日終値からの変化率（％）
    pub fn diff_percent(&self) -> f64 {
        self.diff() / self.previous_close * 100.0
    }
}
