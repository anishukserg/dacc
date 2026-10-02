//! Разбор записей реестра значениями полей — единственное место чтения
//! записей (работа w-access-split). Комментарии снимаются по лексике, строковые
//! литералы не трогаются, значение читается сбалансированной группой до запятой
//! верхнего уровня: форма записи не даёт спрятать значение в комментарий,
//! перенос строки или мультилайн. Идентификатор пути нормализуется в slug:
//! `crate::slice::s_access_contract` и файл `s-access-contract.rs` — одна
//! запись.

/// Поля записи: имя и значение в тексте макроса записи.
pub(crate) fn record_fields(text: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = strip_comments(text).chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut depth = 0i32;
    while i < chars.len() {
        match chars[i] {
            '"' => i = skip_string(&chars, i),
            '(' | '[' | '{' => {
                depth += 1;
                i += 1;
            }
            ')' | ']' | '}' => {
                depth -= 1;
                i += 1;
            }
            c if depth == 1 && (c.is_ascii_alphabetic() || c == '_') => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if chars.get(j) != Some(&':') {
                    continue;
                }
                i = j + 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                let vstart = i;
                let mut level = 0i32;
                while i < chars.len() {
                    match chars[i] {
                        '"' => i = skip_string(&chars, i),
                        '(' | '[' | '{' => {
                            level += 1;
                            i += 1;
                        }
                        ')' | ']' | '}' if level == 0 => break,
                        ')' | ']' | '}' => {
                            level -= 1;
                            i += 1;
                        }
                        ',' if level == 0 => break,
                        _ => i += 1,
                    }
                }
                let value: String = chars[vstart..i].iter().collect();
                out.push((name, value.trim().to_owned()));
            }
            _ => i += 1,
        }
    }
    out
}

/// Текст записи без комментариев: снятие идёт по лексике, строковые литералы
/// не трогаются — значение не спрятать в комментарий.
pub(crate) fn strip_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            let end = skip_string(&chars, i);
            out.extend(&chars[i..end]);
            i = end;
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            // Блочные комментарии вкладываются: счёт глубины снимает весь
            // комментарий, а не до первого `*/` (работа
            // w-record-parsing-hardening).
            i += 2;
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Индекс за закрывающей кавычкой строкового литерала.
fn skip_string(chars: &[char], start: usize) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
        } else if chars[i] == '"' {
            return i + 1;
        } else {
            i += 1;
        }
    }
    i
}

/// Значение поля записи.
pub(crate) fn field_value<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// Первый строковый литерал значения — заголовок и строки записи.
/// Escape-последовательности читаются настоящими символами: `\n` — перевод
/// строки, `\u{…}` — символ по коду (работа w-record-parsing-hardening).
pub(crate) fn string_literal(value: &str) -> Option<String> {
    let start = value.find('"')?;
    let mut out = String::new();
    let mut chars = value[start + 1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                'u' => {
                    if chars.next()? == '{' {
                        let mut hex = String::new();
                        for c in chars.by_ref() {
                            if c == '}' {
                                break;
                            }
                            hex.push(c);
                        }
                        let code = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                        out.push(code?);
                    }
                }
                other => out.push(other),
            },
            '"' => return Some(out),
            c => out.push(c),
        }
    }
    None
}

/// Последний сегмент пути `a::b::c` — идентификатор записи.
pub(crate) fn last_ident(value: &str) -> Option<String> {
    let ident = value
        .rsplit("::")
        .next()?
        .trim()
        .trim_end_matches(')')
        .trim();
    (!ident.is_empty()).then(|| ident.to_owned())
}

/// Последний сегмент пути в виде slug: подчёркивания — дефисы. Ссылка
/// `crate::slice::s_access_contract` и файл `s-access-contract.rs` называют
/// одну запись (работа w-access-split).
pub(crate) fn slug_of(value: &str) -> Option<String> {
    last_ident(value).map(|ident| ident.replace('_', "-"))
}

/// Текст без пробелов: сравнение путей и идентификаторов без формы.
pub(crate) fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Пустой список якорей `&[]` в любой форме записи.
pub(crate) fn anchor_list_is_empty(value: &str) -> bool {
    let flat = flat(value);
    let list = flat.trim_start_matches('&');
    let inner = list
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(list);
    inner.is_empty()
}

/// Заголовок записи: `title: NonEmptyStr::new("…")` в любой форме записи; у
/// записей без поля title (инварианты) заголовком служит statement.
pub(crate) fn title(text: &str) -> String {
    field_string(text, "title")
        .or_else(|| field_string(text, "statement"))
        .unwrap_or_default()
}

/// Строковое значение поля записи: `key: NonEmptyStr::new("…")` — значением.
pub(crate) fn field_string(text: &str, key: &str) -> Option<String> {
    field_value(&record_fields(text), key).and_then(string_literal)
}

/// Ссылка поля в виде slug: `thrust: crate::thrust::t_access_layer` —
/// `t-access-layer`.
pub(crate) fn field_slug(text: &str, key: &str) -> Option<String> {
    field_value(&record_fields(text), key).and_then(slug_of)
}

/// Значение поля текстом, обрезанное по краям.
pub(crate) fn field_text(text: &str, key: &str) -> Option<String> {
    field_value(&record_fields(text), key).map(|value| value.trim().to_owned())
}

/// Статус документа: `status: DocStatus::X` — `X`; без поля статуса — `None`.
pub(crate) fn doc_status(text: &str) -> Option<String> {
    let value = field_text(text, "status")?;
    Some(
        value
            .rsplit("::")
            .next()
            .unwrap_or(&value)
            .trim()
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Значение читается из поля, а не из подстроки: комментарий с тем же
    /// текстом значения не подделывает.
    #[test]
    fn fields_come_from_values_not_from_comments() {
        let text = "work!(\"w-001\",\n    // title: NonEmptyStr::new(\"Фальшь\")\n    title: NonEmptyStr::new(\"Настоящее\"),\n);";
        assert_eq!(title(text), "Настоящее");
    }

    /// Идентификатор пути нормализуется в slug: подчёркивания — дефисы.
    #[test]
    fn path_idents_become_slugs() {
        assert_eq!(
            field_slug(
                "slice!(\"s-001\", thrust: crate::thrust::t_001,);",
                "thrust"
            ),
            Some("t-001".to_owned())
        );
    }

    /// Статус читается значением: форма пути и комментарии не участвуют.
    #[test]
    fn doc_status_reads_the_variant() {
        assert_eq!(
            doc_status("adr!(status: DocStatus::Active,),"),
            Some("Active".to_owned())
        );
        assert_eq!(doc_status("slice!(\"s-001\",),"), None);
    }

    /// Вложенный блочный комментарий снимается целиком: хвост после первого
    /// `*/` не читается как код (работа w-record-parsing-hardening).
    #[test]
    fn nested_comments_are_stripped_whole() {
        let text = "work!(\"w-001\",\n    /* /* хвост */ title: NonEmptyStr::new(\"Фальшь\"), */\n    title: NonEmptyStr::new(\"Настоящее\"),\n);";
        assert_eq!(title(text), "Настоящее");
    }

    /// Escape-последовательности строки читаются символами, а не буквами
    /// (работа w-record-parsing-hardening).
    #[test]
    fn string_literal_reads_escape_sequences() {
        assert_eq!(
            string_literal("NonEmptyStr::new(\"a\\nb\\u{410}\"),"),
            Some("a\nb\u{410}".to_owned())
        );
    }

    /// Запись без поля title показывает statement (работа
    /// w-record-parsing-hardening).
    #[test]
    fn title_falls_back_to_the_statement() {
        let text =
            "invariant!(\"i-001\",\n    statement: NonEmptyStr::new(\"инвариант держится\"),\n);";
        assert_eq!(title(text), "инвариант держится");
    }
}
