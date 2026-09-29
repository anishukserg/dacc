//! Схема слоя работы. Ссылки — пути к константам, порождённым сканом
//! реестров; правило, выразимое типом, валидатором не проверяется.

use dacc_core::{
    axis::Subsystem, AdrRef, BlastRadius, NonEmptyStr, ObligationRef, RfcRef, SliceRef,
    SupersededRef, Taxon, ThrustRef,
};
use std::num::NonZeroU16;

/// Направление: исход на месяцы, ради которого режутся срезы.
#[derive(Debug)]
pub struct Thrust {
    pub id: &'static str,
    pub title: NonEmptyStr,
    /// Наблюдаемый исход. Формулировка обязана быть проверяемой, а не оценочной.
    pub outcome: NonEmptyStr,
}

/// Срез: вертикальное изменение на недели, закрываемое по исходу, а не по календарю.
#[derive(Debug)]
pub struct Slice {
    pub id: &'static str,
    pub title: NonEmptyStr,
    pub thrust: ThrustRef,
    pub outcome: NonEmptyStr,
    /// Спецификация, из которой срез выведен.
    pub specification: RfcRef,
    /// Потолок радиуса для единиц работы среза.
    pub max_radius: BlastRadius,
}

/// Единица работы: от половины дня до недели.
#[derive(Debug)]
pub struct WorkItem {
    pub id: &'static str,
    pub title: NonEmptyStr,
    pub slice: SliceRef,
    pub origin: WorkOrigin,
    /// Ровно одна подсистема: работа на две обязана быть разделена.
    pub taxon: Taxon<Subsystem>,
    pub radius: BlastRadius,
    /// Чем подтверждается готовность. Пока словами: каталог гейтов заменит
    /// это поле ссылками на гейты.
    pub outcome: NonEmptyStr,
}

/// Обязательство: норма без исполнителя или открытый вопрос (решение 35).
///
/// Отдельная запись, а не поле существующей: живёт своим циклом «создано →
/// погашено», не совпадающим с циклом работы. Погашение — приземлённая работа
/// с происхождением [`WorkOrigin::Obligation`].
#[derive(Debug)]
pub struct Obligation {
    pub id: &'static str,
    pub title: NonEmptyStr,
    /// Условие погашения — обязательное: без него запись не компилируется.
    pub discharged_when: NonEmptyStr,
}

/// Происхождение работы: тип задачи и её обоснование — одно поле.
///
/// Перечисление неполное относительно части III: `Migration` появится вместе
/// с реестром, на который ссылается. `Mandate` и `DebtService` той же части
/// реализованы как запись [`Obligation`] и вариант [`WorkOrigin::Obligation`]
/// (решение 35). `#[non_exhaustive]` не ставится намеренно: новый вариант
/// обязан сломать сборку валидатора, а не молча остаться без правила.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkOrigin {
    /// Реализация принятого решения.
    Decision(AdrRef),
    /// Реализация доменной спецификации.
    Specification(RfcRef),
    /// Расхождение фактического с заявленным. Реестра инвариантов с путями
    /// пока нет, поэтому нарушенное утверждение названо текстом рядом со
    /// спецификацией, где оно объявлено.
    Divergence {
        specification: RfcRef,
        violated: NonEmptyStr,
    },
    /// Снятие неопределённости. Поля, объявляющего приземление кода, нет:
    /// исследование, уезжающее в main, невыразимо.
    Inquiry {
        question: NonEmptyStr,
        produces: InquiryOutcome,
        timebox_days: NonZeroU16,
    },
    /// Удаление кода, замещённого решением. Принимает только ссылку
    /// из модуля `superseded`.
    Retirement(SupersededRef),
    /// Погашение обязательства (решение 35): работа, ссылающаяся на
    /// обязательство как на основание. Обязательство погашено, когда эта
    /// работа приземлена.
    Obligation(ObligationRef),
    /// Рутина без архитектурного следа.
    Toil { justification: NonEmptyStr },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InquiryOutcome {
    Adr,
    Rfc,
    Measurement,
}

/// Состояние единицы работы — свёртка журнала (инвариант 3, решение 15), а не
/// поле записи: порождается при сборке реестра из событий.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkState {
    /// Событий нет.
    Planned,
    /// Начата и не завершена.
    Started,
    /// Приземлена с доказательством на том же дереве.
    Landed,
    /// Приземлена до журнала: восстановлена из трейлеров истории.
    LandedFromHistory,
    /// Снята.
    Abandoned,
}

impl WorkState {
    /// После этого состояния событий у работы нет.
    pub const fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Landed | Self::LandedFromHistory | Self::Abandoned
        )
    }
}

impl WorkOrigin {
    /// Приземляет ли работа код. Выводится из варианта, а не хранится.
    pub const fn lands_code(&self) -> bool {
        !matches!(self, Self::Inquiry { .. })
    }
}

/// Радиус работы не выше потолка её среза.
///
/// Функция `const`: скан порождает утверждение на каждую единицу работы,
/// и нарушение становится ошибкой вычисления константы, а не находкой
/// валидатора. Правило связывает две записи, но вычисляется компилятором.
pub const fn radius_within_slice(work: &WorkItem, slices: &[&Slice]) -> bool {
    let mut i = 0;
    while i < slices.len() {
        if slug_eq(slices[i].id, work.slice.as_str()) {
            return work.radius as u8 <= slices[i].max_radius as u8;
        }
        i += 1;
    }
    false
}

/// Побайтовое сравнение slug в `const`: `==` для `&str` нестабилен в константах.
const fn slug_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::NonEmptyStr;

    /// Решение 35: обязательство — отдельная запись, погашение — происхождение
    /// работы, которое приземляет код.
    #[test]
    fn obligation_record_and_origin() {
        let obligation = Obligation {
            id: "o-fix-x",
            title: NonEmptyStr::new("Перейти на …"),
            discharged_when: NonEmptyStr::new("когда …"),
        };
        assert_eq!(obligation.id, "o-fix-x");
        assert_eq!(obligation.discharged_when.as_str(), "когда …");

        let origin = WorkOrigin::Obligation(ObligationRef::__from_scan("o-fix-x"));
        assert!(origin.lands_code());
    }
}
