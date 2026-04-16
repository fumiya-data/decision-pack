//! 複数の列モジュールで使う文字列・日付正規化補助です。

use chrono::NaiveDate;

/// 全角文字や一部記号の揺れを ASCII 相当へ正規化します。
///
/// 元データには通常の ASCII、BOM、全角数字、日本語環境由来の記号揺れが
/// 混在します。共有ヘルパーでまとめて吸収することで、各列の処理は業務規則へ
/// 集中できます。
pub fn normalize_width(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '\u{feff}' => {}
            '\u{3000}' => out.push(' '),
            '\u{2019}' => out.push('\''),
            // ID、電話番号、郵便番号で頻出するダッシュ類をまとめて正規化し、
            // 後続の検証が同一扱いできるようにします。
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}'
            | '\u{ff0d}' => out.push('-'),
            // 全角 ASCII は連続したコード帯にあるため、個別列挙ではなく
            // オフセット計算でまとめて変換します。
            '\u{ff01}'..='\u{ff5e}' => {
                let mapped = char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch);
                out.push(mapped);
            }
            _ => out.push(ch),
        }
    }

    out
}

/// 連続する空白を半角スペース 1 つへ畳み、前後も取り除きます。
pub fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 文字幅正規化のあとに空白畳み込みを適用します。
pub fn clean_text(value: &str) -> String {
    collapse_whitespace(&normalize_width(value))
}

/// 複数行テキストを整形済みの 1 行へ畳み込みます。
///
/// 汚れたエクスポートでは `notes` に改行が混入します。内容は保持しつつ、
/// 各行を正規化して空白連結した 1 行へ変換します。
pub fn clean_multiline_text(value: &str) -> String {
    normalize_width(value)
        .lines()
        .map(collapse_whitespace)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// よくある欠損プレースホルダであり、意味のある値ではない場合に `true` を返します。
pub fn is_placeholder(value: &str) -> bool {
    matches!(
        clean_text(value).to_ascii_lowercase().as_str(),
        "null" | "unknown" | "na" | "n/a" | "-" | "none"
    )
}

/// 氏名や地名に向く保守的なタイトルケース変換を適用します。
///
/// 空白やアポストロフィ、ハイフンの直後を大文字化するため、
/// `O'Neil` や `Anne-Marie` のような形も保ちやすくしています。
pub fn title_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut uppercase_next = true;

    for ch in value.chars() {
        if ch.is_whitespace() {
            out.push(' ');
            uppercase_next = true;
            continue;
        }

        if matches!(ch, '\'' | '-' | '.' | '/' | '’') {
            out.push(ch);
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }

    out
}

/// カンマ区切りの断片を一貫した空白ルールで並べ直します。
pub fn normalize_comma_spacing(value: &str) -> String {
    value
        .split(',')
        .map(collapse_whitespace)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

/// 顧客エクスポートに出現する限られた日付表記を解析します。
///
/// このファイルには ISO 日付、スラッシュ区切り、日先頭のハイフン区切り、
/// タイムスタンプ混在が含まれます。未対応形式を黙って推測せず、日付部分だけを
/// 抽出して小さなヒューリスティクスで判定します。
pub fn parse_flexible_date(value: &str) -> Result<NaiveDate, String> {
    let cleaned = clean_text(value);
    if cleaned.is_empty() {
        return Err("blank date".to_string());
    }

    let date_only = cleaned.split_whitespace().next().unwrap_or("");
    if date_only.is_empty() {
        return Err("blank date".to_string());
    }

    if let Ok(date) = NaiveDate::parse_from_str(date_only, "%Y-%m-%d") {
        return Ok(date);
    }

    if let Ok(date) = NaiveDate::parse_from_str(date_only, "%Y/%m/%d") {
        return Ok(date);
    }

    // レガシー表記では `-` が日先頭、`/` が月先頭に使われています。
    // 年先頭の表記はそれより前で処理済みです。
    let separator = if date_only.contains('-') {
        '-'
    } else if date_only.contains('/') {
        '/'
    } else {
        return Err("unsupported date separator".to_string());
    };

    let parts: Vec<&str> = date_only.split(separator).collect();
    if parts.len() != 3 {
        return Err("expected three date parts".to_string());
    }

    if parts[0].len() == 4 {
        let year = parts[0]
            .parse::<i32>()
            .map_err(|_| "invalid year".to_string())?;
        let month = parts[1]
            .parse::<u32>()
            .map_err(|_| "invalid month".to_string())?;
        let day = parts[2]
            .parse::<u32>()
            .map_err(|_| "invalid day".to_string())?;
        return NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "date is out of range".to_string());
    }

    if parts[2].len() != 4 {
        return Err("unsupported year placement".to_string());
    }

    let year = parts[2]
        .parse::<i32>()
        .map_err(|_| "invalid year".to_string())?;
    let (month, day) = if separator == '/' {
        (
            parts[0]
                .parse::<u32>()
                .map_err(|_| "invalid month".to_string())?,
            parts[1]
                .parse::<u32>()
                .map_err(|_| "invalid day".to_string())?,
        )
    } else {
        (
            parts[1]
                .parse::<u32>()
                .map_err(|_| "invalid month".to_string())?,
            parts[0]
                .parse::<u32>()
                .map_err(|_| "invalid day".to_string())?,
        )
    };

    NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "date is out of range".to_string())
}
