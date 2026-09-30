//! Схема решения. Правила, которые в обычной методологии проверялись бы
//! валидатором, здесь выражены типами и потому не проверяются вовсе.
//! Атаки на эти правила и их контроли — [`crate::attacks`].

use chrono::NaiveDate;
use dacc_core::{axis::Subsystem, AdrRef, AnchorId, Channel, NonEmpty, NonEmptyStr, RfcRef, Taxon};

/// Статус документа.
///
/// `SupersededBy` несёт ссылку на замещающее решение: замещение без указания,
/// чем именно замещено, невыразимо.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocStatus {
    Draft,
    Active,
    Deprecated,
    SupersededBy(AdrRef),
}

/// Ломающее изменение и инструкции миграции — одна конструкция.
///
/// Приём из решения 4: прежде здесь была пара полей
/// `breaking: bool` + `migration_notes: &[&str]`, согласованность которых
/// приходилось сверять валидатором. Теперь инструкции живут **внутри**
/// варианта `Yes` и непусты по типу, поэтому отметить изменение ломающим
/// и не приложить инструкции нельзя синтаксически. Каждый шаг тоже непуст:
/// список пустых строк формально непуст, но ничего не предписывает.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breaking {
    No,
    Yes { migration: NonEmpty<NonEmptyStr> },
}

impl Breaking {
    pub const fn is_breaking(&self) -> bool {
        matches!(self, Self::Yes { .. })
    }
}

#[derive(Debug)]
pub struct ArchitectureDecision {
    pub title: &'static str,
    pub status: DocStatus,
    /// Только значения оси подсистем: значение другой оси — ошибка типа.
    pub subsystems: &'static [Taxon<Subsystem>],
    /// Канал публикации документа (решение 21): публикуется только документ
    /// своего канала; ссылка через границу канала — отказ экспорта.
    pub channel: Channel,
    pub context: &'static str,
    pub decision: &'static str,
    pub trade_offs: &'static [&'static str],
    pub constraints: &'static [&'static str],
    /// Непусто по типу, и каждое имя непусто: запись без авторов невыразима.
    pub authors: NonEmpty<NonEmptyStr>,
    pub decided_at: NaiveDate,
    pub breaking: Breaking,
    /// Ссылки на разметку кода. Каждая — путь к константе, порождённой
    /// сканом исходников: удалили разметку, ссылка не собирается.
    pub code_refs: &'static [AnchorId],
    pub related_rfcs: &'static [RfcRef],
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::nonempty_str;

    #[test]
    fn breaking_carries_its_migration() {
        let b = Breaking::Yes {
            migration: nonempty_str!["перекодировать страницы"],
        };
        assert!(b.is_breaking());
        assert_eq!(
            b,
            Breaking::Yes {
                migration: nonempty_str!["перекодировать страницы"]
            }
        );
    }

    #[test]
    fn superseded_names_its_successor() {
        let s = DocStatus::SupersededBy(AdrRef::__from_scan("adr-2026-007"));
        assert!(matches!(s, DocStatus::SupersededBy(r) if r.as_str() == "adr-2026-007"));
    }
}

/// Доменная спецификация: что требуется, в отличие от решения — как сделано.
///
/// Отвечает на вопрос «чего мы хотим», решение — на вопрос «как решили».
/// Работа выводится либо из спецификации, либо из решения.
#[derive(Debug)]
pub struct DomainSpecification {
    pub title: &'static str,
    pub status: DocStatus,
    pub target: &'static [Taxon<Subsystem>],
    /// Канал публикации спецификации (решение 21).
    pub channel: Channel,
    /// Наблюдаемый результат. Формулировка обязана быть проверяемой
    /// инструментом, а не оценочной.
    pub goal: &'static str,
    pub input_contract: &'static str,
    pub output_contract: &'static str,
    /// Утверждения, которые обязаны держаться. Непусто по типу: спецификация
    /// без единого инварианта ничего не требует.
    pub invariants: NonEmpty<NonEmptyStr>,
    pub authors: NonEmpty<NonEmptyStr>,
    pub decided_at: NaiveDate,
    /// Решения, реализующие эту спецификацию.
    pub decided_by: &'static [AdrRef],
}

/// Инвариант: гарантия, которая обязана держаться (решение 45). Статус —
/// вариант, а не строка: запись не может сказать «Enforced», не сказав, как
/// нарушение было проведено.
#[derive(Debug)]
pub struct Invariant {
    pub id: &'static str,
    /// Как далеко инвариант доведён.
    pub status: InvariantStatus,
    /// Что обязано держаться.
    pub statement: NonEmptyStr,
    /// Почему обязано держаться и как выглядит нарушение.
    pub rationale: &'static str,
    /// Разметка кода, где инвариант держится.
    pub enforced_by: &'static [AnchorId],
    /// Разметка тестов, которые его доказывают.
    pub tests: &'static [AnchorId],
}

/// Как далеко доведён инвариант (решение 45).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantStatus {
    /// Решено, не построено: кода, тестов и обхода нет.
    Planned,
    /// Код и тесты есть; обход описан, но не опробован.
    Claimed { bypass: NonEmptyStr },
    /// Код, тесты и попытка обхода существуют.
    Enforced(Enforced),
}

/// Проверенный инвариант: нарушение невыразимо или обход опробован тестами.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enforced {
    /// Типы делают нарушение невыразимым.
    Unrepresentable,
    /// Описанный обход опробован атакующими тестами.
    Adversarial {
        bypass: NonEmptyStr,
        tests: &'static [AnchorId],
    },
}

/// Независимое чтение (решение 46): угол, находки и их судьба.
#[derive(Debug)]
pub struct Review {
    pub id: &'static str,
    /// Что прочитано.
    pub title: NonEmptyStr,
    /// Угол, под которым читали, — граница чтения на записи.
    pub angle: NonEmptyStr,
    /// Каждая находка и что с ней стало.
    pub findings: NonEmpty<NonEmptyStr>,
    /// Кто читал; для вердикта — читатель, не пишущий других записей.
    pub authors: NonEmpty<NonEmptyStr>,
    /// Когда чтение состоялось.
    pub decided_at: NaiveDate,
}
