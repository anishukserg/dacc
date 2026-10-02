//! Ратчет миграции проверок (решение 40, работа w-declare-baseline):
//! `declare_baseline!` объявляет известный долг — список нарушений с именем
//! и замороженной мерой, — а [`ratchet`] сверяет с ним текущие нарушения
//! проверки. Нарушение вне baseline и любое отклонение меры — ошибка сборки с
//! именем нарушения: baseline обновляется только явным решением — уменьшение
//! фиксируется явно, рост не легализуется молча.

/// Запись baseline: имя нарушения и замороженная мера.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// Имя нарушения — стабильный идентификатор, а не текст.
    pub name: &'static str,
    /// Замороженная мера нарушения на момент объявления.
    pub measure: usize,
}

/// Текущее нарушение проверки: имя и мера.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Имя нарушения — то же, что в [`Entry`].
    pub name: String,
    /// Текущая мера нарушения.
    pub measure: usize,
}

/// Объявляет baseline: список известных нарушений с именем и мерой.
///
/// ```text
/// dacc_scan::declare_baseline!(
///     "crates/tool/src/gate.rs" => 1458,
///     "crates/tool/src/work.rs" => 1360,
/// );
/// ```
#[macro_export]
macro_rules! declare_baseline {
    ($($name:literal => $measure:expr),* $(,)?) => {
        &[$($crate::baseline::Entry { name: $name, measure: $measure }),*]
    };
}

/// Сверяет текущие нарушения проверки с baseline (решение 40).
///
/// Нарушение вне baseline — ошибка: новый долг вносится в baseline явным
/// решением. Мера нарушения, названного в baseline, обязана совпадать с
/// заморозкой в точности: уменьшение фиксируется явным обновлением записи,
/// рост не легализуется молча. Запись без нарушения — погашенный долг: она
/// убирается явным решением, а не остаётся в baseline.
#[dacc_derive::doc_anchor(id = "baseline-ratchet")]
pub fn ratchet(current: &[Violation], baseline: &[Entry]) -> Result<(), String> {
    for violation in current {
        match baseline.iter().find(|entry| entry.name == violation.name) {
            None => {
                return Err(format!(
                    "нарушение вне baseline: {} {} — внесите в baseline отдельным решением",
                    violation.name, violation.measure
                ))
            }
            Some(entry) if violation.measure != entry.measure => {
                return Err(format!(
                    "мера нарушения изменилась молча: {} {} != {} — обновите baseline отдельным решением",
                    violation.name, violation.measure, entry.measure
                ))
            }
            Some(_) => {}
        }
    }
    for entry in baseline {
        if !current.iter().any(|violation| violation.name == entry.name) {
            return Err(format!(
                "baseline называет погашенное нарушение: {} — уберите запись отдельным решением",
                entry.name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROZEN: usize = 10;

    fn baseline() -> [Entry; 1] {
        [Entry {
            name: "known",
            measure: FROZEN,
        }]
    }

    fn violation(measure: usize) -> Vec<Violation> {
        vec![Violation {
            name: "known".to_owned(),
            measure,
        }]
    }

    /// Baseline держит текущие нарушения: заморозка проходит сверку.
    #[test]
    fn the_baseline_freezes_the_current_violations() {
        assert!(ratchet(&violation(FROZEN), &baseline()).is_ok());
    }

    /// Рост меры сверх заморозки — ошибка сборки с именем нарушения.
    #[test]
    fn growth_beyond_the_baseline_is_an_error() {
        let problem = ratchet(&violation(FROZEN + 1), &baseline()).expect_err("рост ожидается");
        assert!(problem.contains("known"), "{problem}");
        assert!(problem.contains("изменилась молча"), "{problem}");
    }

    /// Уменьшение меры — тоже ошибка: оно фиксируется явным обновлением
    /// записи, а не сверкой (работа w-declare-baseline).
    #[test]
    fn shrinking_needs_an_explicit_baseline_edit() {
        let problem =
            ratchet(&violation(FROZEN - 1), &baseline()).expect_err("уменьшение ожидается");
        assert!(problem.contains("known"), "{problem}");
        assert!(problem.contains("обновите baseline"), "{problem}");
    }

    /// Нарушение вне baseline и запись без нарушения — ошибки решений.
    #[test]
    fn new_violations_and_stale_entries_are_refused() {
        let fresh = vec![Violation {
            name: "new".to_owned(),
            measure: 1,
        }];
        let problem = ratchet(&fresh, &baseline()).expect_err("новое нарушение ожидается");
        assert!(problem.contains("вне baseline"), "{problem}");

        let problem = ratchet(&[], &baseline()).expect_err("погашенная запись ожидается");
        assert!(problem.contains("погашенное нарушение"), "{problem}");
    }
}
