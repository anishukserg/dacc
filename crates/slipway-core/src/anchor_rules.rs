//! Правила записи slug-идентификаторов: общие для разметки кода
//! (`#[doc_anchor]`), скана разметки и идентификаторов документов знания.
//!
//! Одно место на всех исполнителей: атрибут отвергает ошибку при сборке
//! продукта, скан — при сборке реестра. Разойтись им не в чем.

/// Допустимые режимы отображения фрагмента.
pub const ANCHOR_MODES: [&str; 3] = ["ref", "embed", "snippet"];

/// Идентификатор-«slug»: латиница в нижнем регистре, цифры и дефис,
/// начинается с буквы. Общее правило для разметки кода и идентификаторов
/// документов знания.
///
/// Отображается в имя константы заменой дефиса на подчёркивание, поэтому
/// подчёркивание в самом идентификаторе запрещено (иначе `plan-ir` и
/// `plan_ir` дали бы одну константу), а ключевые слова Rust — тоже.
pub fn check_slug_id(id: &str) -> Result<(), String> {
    if !id.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Err(format!(
            "slug id {id:?} must start with a lowercase latin letter"
        ));
    }
    if let Some(bad) = id
        .chars()
        .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
    {
        return Err(format!(
            "slug id {id:?}: character {bad:?} is not allowed; allowed are a-z, 0-9 and a hyphen"
        ));
    }
    if KEYWORDS.contains(&slug_ident(id).as_str()) {
        return Err(format!("slug id {id:?} is a Rust keyword"));
    }
    Ok(())
}

/// Режим обязан быть одним из [`ANCHOR_MODES`]: неизвестный режим не
/// подменяется молча режимом по умолчанию.
pub fn check_anchor_mode(mode: &str) -> Result<(), String> {
    if ANCHOR_MODES.contains(&mode) {
        Ok(())
    } else {
        Err(format!(
            "anchor mode {mode:?} is unknown; allowed are ref, embed, snippet"
        ))
    }
}

/// Имя порождаемой константы для slug-идентификатора.
pub fn slug_ident(id: &str) -> String {
    id.replace('-', "_")
}

const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_kebab_case_ids() {
        for ok in ["plan-ir", "wal-record-v3", "adr-2026-001", "a"] {
            assert!(check_slug_id(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn rejects_ids_that_cannot_be_constants() {
        for bad in ["", "2pc", "plan.ir", "Plan-IR", "type", "self", "plan_ir"] {
            assert!(check_slug_id(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn mode_is_closed_set() {
        assert!(check_anchor_mode("snippet").is_ok());
        assert!(check_anchor_mode("snipet").is_err());
    }
}
