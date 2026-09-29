//! Плоское подмножество TOML: пустые строки, комментарии `# …` и пары
//! `ключ = "строка"`. Таблиц, массивов, чисел и многострочных строк журнал не
//! использует, а парсер без зависимостей обязан быть строгим: всё, что за
//! пределами подмножества, — ошибка с номером строки.

/// Запись файла журнала: пары ключ — значение в порядке файла.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    pub fields: Vec<(String, String)>,
}

impl Record {
    /// Значение ключа.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

/// Ошибка разбора: номер строки с единицы и причина.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    pub line: usize,
    pub reason: String,
}

/// Разбирает текст файла события.
pub fn parse(text: &str) -> Result<Record, FormatError> {
    let mut record = Record::default();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parse_pair(&mut record, line, index + 1)?;
    }
    Ok(record)
}

/// Разбирает одну запись журнала (решение 43): массив таблиц `[[events]]`, по
/// одной таблице на событие. Запись без таблиц — ошибка.
pub fn parse_events(text: &str) -> Result<Vec<Record>, FormatError> {
    let mut records = Vec::new();
    let mut current: Option<Record> = None;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line_no = index + 1;
        if line == "[[events]]" {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some(Record::default());
            continue;
        }
        let record = current.as_mut().ok_or_else(|| FormatError {
            line: line_no,
            reason: "expected `[[events]]` before the first event".to_owned(),
        })?;
        parse_pair(record, line, line_no)?;
    }
    if let Some(record) = current {
        records.push(record);
    }
    if records.is_empty() {
        return Err(FormatError {
            line: 1,
            reason: "the journal has no events".to_owned(),
        });
    }
    Ok(records)
}

fn parse_pair(record: &mut Record, line: &str, line_no: usize) -> Result<(), FormatError> {
    let fail = |reason: String| FormatError {
        line: line_no,
        reason,
    };
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| fail("expected `key = \"string\"`".to_owned()))?;
    let key = key.trim();
    if !is_key(key) {
        return Err(fail(format!(
            "key `{key}` is lowercase latin letters, digits and `_`, starting with a letter"
        )));
    }
    if record.get(key).is_some() {
        return Err(fail(format!("key `{key}` is repeated")));
    }
    let value = parse_string(value.trim()).map_err(fail)?;
    record.fields.push((key.to_owned(), value));
    Ok(())
}

/// Текст записи: пары в заданном порядке. Управляющие символы, кроме перевода
/// строки и табуляции, заменяются пробелом: подмножество их не допускает.
pub fn render(fields: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let mut out = String::new();
    for (key, value) in fields {
        out.push_str(key.as_ref());
        out.push_str(" = \"");
        for c in value.as_ref().chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push(' '),
                c => out.push(c),
            }
        }
        out.push_str("\"\n");
    }
    out
}

fn is_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn parse_string(text: &str) -> Result<String, String> {
    let inner = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .ok_or_else(|| {
            "a value is a double-quoted string and nothing else on the line".to_owned()
        })?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                _ => return Err("a string allows only the escapes \\\" \\\\ \\n \\t".to_owned()),
            },
            '"' => return Err("an unescaped quote inside a string".to_owned()),
            c if c.is_control() => return Err("a control character in a string".to_owned()),
            c => out.push(c),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_comments_are_read_in_order() {
        let record = parse("# событие\nevent = \"started\"\n\nwork = \"w0001\"\n").unwrap();
        assert_eq!(record.get("event"), Some("started"));
        assert_eq!(record.fields[1], ("work".to_owned(), "w0001".to_owned()));
    }

    #[test]
    fn render_and_parse_round_trip_escapes() {
        let text = render(&[("reason", "кавычка \" и \\ и\nперевод")]);
        assert_eq!(
            parse(&text).unwrap().get("reason"),
            Some("кавычка \" и \\ и\nперевод")
        );
    }

    #[test]
    fn anything_outside_the_subset_names_its_line() {
        for (text, line, reason) in [
            ("event = started", 1, "double-quoted"),
            ("\n[table]", 2, "expected"),
            ("Event = \"x\"", 1, "lowercase"),
            ("a = \"x\"\na = \"y\"", 2, "repeated"),
            ("a = 1", 1, "double-quoted"),
            ("a = \"x\" # хвост", 1, "double-quoted"),
            ("a = \"x\\q\"", 1, "only the escapes"),
            ("a = \"x\"y\"", 1, "unescaped quote"),
        ] {
            let error = parse(text).expect_err(text);
            assert_eq!(error.line, line, "{text}");
            assert!(error.reason.contains(reason), "{text}: {}", error.reason);
        }
    }
}
