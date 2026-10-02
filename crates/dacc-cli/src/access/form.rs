//! Форма строк ответов: JSON — [`crate::format::string`], XML —
//! [`xml_string`], потолки ответов — [`cap_text`]. Один экземпляр каждой
//! формы на инструмент (работа w-access-split).

/// Строка JSON — единственная форма экранирования инструмента
/// [`crate::format::string`]: управляющие символы и метасимволы обрабатываются
/// в одном месте.
pub(crate) use crate::format::string as json_string;

/// Потолок ответа map (RFC-0003): 8 КБ независимо от размера проекта.
pub(crate) const MAP_LIMIT: usize = 8192;

/// Потолок ответа state (RFC-0003): 2 КБ.
pub(crate) const STATE_LIMIT: usize = 2048;

/// Строка XML с экранированием разметки в атрибутах (работа
/// w-xml-wellformed): запрещённые управляющие символы XML 1.0 заменяются
/// пробелом — как вывод журнала, — чтобы ответ оставался пригодным парсеру.
pub(crate) fn xml_string(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '&' => out.push_str(concat!("&", "amp;")),
            '<' => out.push_str(concat!("&", "lt;")),
            '>' => out.push_str(concat!("&", "gt;")),
            '"' => out.push_str(concat!("&", "quot;")),
            '\'' => out.push_str(concat!("&", "apos;")),
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Явное усечение текста до потолка: ответ не превышает limit, и усечение
/// названо строкой, а не молчанием.
pub(crate) fn cap_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let total = text.lines().count();
    let mut out = String::new();
    for (index, line) in text.lines().enumerate() {
        if out.len() + line.len() + 64 > limit {
            out.push_str(&format!("truncated: {index} of {total} lines\n"));
            return out;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Ответ сбоя машинному контракту (работа w-legitimate-failure): законный
/// отказ отличается от сбоя полем legitimate — сбой отвечает, а не падает
/// молча.
pub(crate) fn error_json(reason: &str) -> String {
    format!(
        "{{\"schema\": \"dacc-error\", \"legitimate\": false, \"reason\": {}}}",
        json_string(reason)
    )
}
