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

    /// 単純移動平均（SMA）。history と同じ長さで、i 番目には
    /// 「i 番目を含む直近 period 件の平均」が入る。
    ///
    /// period 件そろわない先頭は None になる。
    /// 合計を持ち回して1件ずつ出し入れするため、計算量は履歴の件数に比例する。
    pub fn moving_average(&self, period: usize) -> Vec<Option<f64>> {
        if period == 0 {
            return vec![None; self.history.len()];
        }

        let mut averages = Vec::with_capacity(self.history.len());
        let mut sum = 0.0;

        for (i, day) in self.history.iter().enumerate() {
            sum += day.close;

            // 期間から外れた分を引く
            if i >= period {
                sum -= self.history[i - period].close;
            }

            if i + 1 >= period {
                averages.push(Some(sum / period as f64));
            } else {
                averages.push(None);
            }
        }

        averages
    }
}

#[cfg(test)]
mod tests {
    use super::{DailyClose, Stock};
    use chrono::{FixedOffset, TimeZone};

    /// 終値だけを持つ Stock を組み立てる（移動平均の検証用）
    fn stock_with_closes(closes: &[f64]) -> Stock {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();

        Stock {
            name: "テスト".to_string(),
            symbol: "TEST".to_string(),
            currency: "JPY".to_string(),
            time: jst.timestamp_opt(0, 0).unwrap(),
            price: 0.0,
            previous_close: 0.0,
            day_high: 0.0,
            day_low: 0.0,
            volume: 0,
            history: closes
                .iter()
                .map(|close| DailyClose {
                    date: jst.timestamp_opt(0, 0).unwrap(),
                    close: *close,
                })
                .collect(),
        }
    }

    #[test]
    fn 期間がそろうまでは値を出さない() {
        let stock = stock_with_closes(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let actual = stock.moving_average(3);

        assert_eq!(actual, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
    }

    #[test]
    fn 履歴より長い期間を指定すると値が出ない() {
        let stock = stock_with_closes(&[1.0, 2.0]);

        assert_eq!(stock.moving_average(25), vec![None, None]);
    }

    #[test]
    fn 期間が1なら終値そのものになる() {
        let stock = stock_with_closes(&[10.0, 20.0]);

        assert_eq!(stock.moving_average(1), vec![Some(10.0), Some(20.0)]);
    }

    #[test]
    fn 履歴が空なら結果も空() {
        let stock = stock_with_closes(&[]);

        assert!(stock.moving_average(25).is_empty());
    }
}
