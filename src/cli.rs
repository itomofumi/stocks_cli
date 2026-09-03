//! コマンドライン引数の定義。
//!
//! derive(Parser) を付けると、この struct から clap が自動でパーサを生成する。
//! ドキュメンテーションコメント（///）がそのまま --help の説明文になる。

use clap::Parser;

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
pub struct Args {
    /// 銘柄コード。空白区切りで複数指定できる
    #[arg(default_value = "7203.T")]
    pub symbols: Vec<String>,

    /// 取得期間
    #[arg(short, long, default_value = "5d")]
    pub range: String,

    /// グラフを表示しない
    #[arg(long)]
    pub no_chart: bool,
}
