//! Формат вывода вердикта (решение 22).
//!
//! Человекочитаемый вывод остаётся выводом по умолчанию; машинный включается
//! флагом `--format json`. Машинный вывод — JSON полями, чтобы потребитель
//! читал структуру, а не разбирал предложение.

use std::ffi::OsString;
use std::fmt::Write as _;

/// Формат вывода команды, выносящей вердикт.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    /// Человекочитаемый вывод (по умолчанию).
    Text,
    /// Машинный вывод: JSON полями.
    Json,
}

/// Разбор значения `--format`. Ошибка называет допустимые значения.
pub fn parse(value: &str) -> Result<Format, String> {
    match value {
        "json" => Ok(Format::Json),
        "text" => Ok(Format::Text),
        other => Err(format!("--format needs `json` or `text`, not `{other}`")),
    }
}

/// Формат из аргументов: `--format json|text` в любом месте. Для негодного
/// значения или отсутствия — `Text`: ошибку самого флага печатает разборщик
/// аргументов команды, а она обязана остаться человекочитаемой.
pub fn scan(args: &[OsString]) -> Format {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg.to_str() == Some("--format") {
            if let Some(format) = iter
                .next()
                .and_then(|value| value.to_str())
                .and_then(|value| parse(value).ok())
            {
                return format;
            }
        }
    }
    Format::Text
}

/// Строка JSON в двойных кавычках, с экранированием.
pub fn string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_json_and_text() {
        assert_eq!(parse("json"), Ok(Format::Json));
        assert_eq!(parse("text"), Ok(Format::Text));
        assert!(parse("xml").is_err());
    }

    #[test]
    fn scan_reads_the_flag_anywhere() {
        let args = |words: &[&str]| words.iter().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(scan(&args(&["--format", "json"])), Format::Json);
        assert_eq!(scan(&args(&["state", "--format", "json"])), Format::Json);
        assert_eq!(scan(&args(&["--format", "json", "state"])), Format::Json);
        assert_eq!(scan(&args(&["state"])), Format::Text);
        // Негодное значение флага — Text: ошибку печатает разборщик команды.
        assert_eq!(scan(&args(&["--format", "xml"])), Format::Text);
    }

    #[test]
    fn string_escapes_quotes_backslashes_and_control() {
        assert_eq!(string("plain"), "\"plain\"");
        assert_eq!(string("a\"b"), "\"a\\\"b\"");
        assert_eq!(string("a\\b"), "\"a\\\\b\"");
        assert_eq!(string("a\nb"), "\"a\\nb\"");
        assert_eq!(string("a\tb"), "\"a\\tb\"");
    }
}
