//! コマンドライン引数の定義。
//!
//! derive(Parser) を付けると、この struct から clap が自動でパーサを生成する。
//! ドキュメンテーションコメント（///）がそのまま --help の説明文になる。

use clap::{CommandFactory, Parser, ValueEnum};
use std::collections::HashSet;

/// 一度に指定できる銘柄数の上限。
///
/// 銘柄ごとに1回ずつ逐次リクエストするため、数が多いと
/// Yahoo からレート制限を受ける恐れがある。
pub const MAX_SYMBOLS: usize = 20;

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
  ※ 一度に指定できるのは20銘柄までです

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

impl Args {
    /// 引数を読み込み、clap だけでは表現できない制約を検証する。
    ///
    /// 件数の上限は num_args でも書けるが、その場合のメッセージは
    /// 「no more were expected」となり上限値が伝わらない。
    /// Command::error() を使うと、clap 本来の書式のまま文面を指定できる。
    pub fn parse_and_validate() -> Self {
        let mut args = Self::parse();

        args.symbols = normalize_symbols(args.symbols);

        // 空文字だけを渡された場合、除去の結果1件も残らない
        if args.symbols.is_empty() {
            Self::command()
                .error(
                    clap::error::ErrorKind::InvalidValue,
                    "銘柄コードが1件も指定されていません",
                )
                .exit();
        }

        // 件数の確認は除去後に行う。同じ銘柄を並べただけで
        // 上限に達するのは不親切なため。
        if args.symbols.len() > MAX_SYMBOLS {
            Self::command()
                .error(
                    clap::error::ErrorKind::TooManyValues,
                    format!(
                        "銘柄は一度に{}件までです（{}件指定されました）",
                        MAX_SYMBOLS,
                        args.symbols.len()
                    ),
                )
                .exit(); // 終了コード 2 で終了する
        }

        args
    }
}

/// 空文字を取り除き、重複を1つにまとめる。
///
/// 指定順は保ち、先に現れた方を残す。
/// Yahoo は銘柄コードの大文字小文字を区別しない（aapl でも AAPL のデータが返る）ため、
/// 比較は大文字に揃えて行う。
fn normalize_symbols(symbols: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    symbols
        .into_iter()
        .filter(|symbol| !symbol.trim().is_empty())
        // insert は「新しく追加できたか」を返す。既出なら false になり除外される
        .filter(|symbol| seen.insert(symbol.to_uppercase()))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::normalize_symbols;

    fn symbols(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn 重複を除去し指定順を保つ() {
        let actual = normalize_symbols(symbols(&["6758.T", "7203.T", "6758.T"]));
        assert_eq!(actual, symbols(&["6758.T", "7203.T"]));
    }

    #[test]
    fn 大文字小文字を同一視し先に現れた表記を残す() {
        let actual = normalize_symbols(symbols(&["AAPL", "aapl", "Aapl"]));
        assert_eq!(actual, symbols(&["AAPL"]));
    }

    #[test]
    fn 空文字と空白のみの要素を除去する() {
        let actual = normalize_symbols(symbols(&["", "7203.T", "   "]));
        assert_eq!(actual, symbols(&["7203.T"]));
    }

    #[test]
    fn 除去対象がなければそのまま返す() {
        let actual = normalize_symbols(symbols(&["7203.T", "6758.T"]));
        assert_eq!(actual, symbols(&["7203.T", "6758.T"]));
    }
}
