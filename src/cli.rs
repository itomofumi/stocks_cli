//! コマンドライン引数の定義。
//!
//! derive(Parser) を付けると、この struct から clap が自動でパーサを生成する。
//! ドキュメンテーションコメント（///）がそのまま --help の説明文になる。

use clap::{Parser, ValueEnum};

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

例:
  stocks_cli                      トヨタを直近5営業日分（既定）
  stocks_cli 6758.T               ソニーを表示
  stocks_cli 7203.T 6758.T AAPL   複数銘柄をまとめて表示
  stocks_cli AAPL -r 1mo          アップルを1か月分
  stocks_cli ^N225 --no-chart     日経平均をグラフなしで表示

終了コード:
  0 = 全銘柄の取得に成功   1 = 1銘柄でも失敗（エラーは標準エラー出力へ）"
)]
pub struct Args {
    /// 銘柄コード。空白区切りで複数指定できる
    #[arg(default_value = "7203.T")]
    pub symbols: Vec<String>,

    /// 取得期間
    #[arg(short, long, default_value = "5d")]
    pub range: Range,

    /// グラフを表示しない
    #[arg(long)]
    pub no_chart: bool,
}

/// 取得期間。Yahoo が受け付ける値だけを列挙する。
///
/// derive(ValueEnum) により、clap が引数のパース時に検証してくれる。
/// 一覧はヘルプに possible values として自動表示される。
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Range {
    // 変種名は数字から始められないため、コマンドラインでの表記は value(name) で指定する
    #[value(name = "1d")]
    OneDay,
    #[value(name = "5d")]
    FiveDays,
    #[value(name = "1mo")]
    OneMonth,
    #[value(name = "3mo")]
    ThreeMonths,
    #[value(name = "6mo")]
    SixMonths,
    #[value(name = "ytd")]
    Ytd,
    #[value(name = "1y")]
    OneYear,
    #[value(name = "2y")]
    TwoYears,
    #[value(name = "5y")]
    FiveYears,
}

impl Range {
    /// API に渡す文字列（＝コマンドラインでの表記）
    pub fn as_str(&self) -> &'static str {
        match self {
            Range::OneDay => "1d",
            Range::FiveDays => "5d",
            Range::OneMonth => "1mo",
            Range::ThreeMonths => "3mo",
            Range::SixMonths => "6mo",
            Range::Ytd => "ytd",
            Range::OneYear => "1y",
            Range::TwoYears => "2y",
            Range::FiveYears => "5y",
        }
    }
}
