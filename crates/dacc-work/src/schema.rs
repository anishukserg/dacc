//! Схема слоя работы. Ссылки — пути к константам, порождённым сканом
//! реестров; правило, выразимое типом, валидатором не проверяется.

use dacc_core::{
    axis::Subsystem, AdrRef, BlastRadius, NonEmptyStr, ObligationRef, RfcRef, SliceRef,
    SupersededRef, Taxon, ThrustRef, WorkRef,
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

/// Каждое обязательство погашено приземлённой работой с происхождением
/// [`WorkOrigin::Obligation`] (решение 35). Функция `const`: скан порождает
/// утверждение, и нарушение — ошибка вычисления константы, а не находка
/// валидатора. Так молчаливое забывание обязательства невыразимо.
pub const fn obligations_redeemed(
    obligations: &[(&str, &Obligation)],
    work: &[&WorkItem],
    states: &[(WorkRef, WorkState)],
) -> bool {
    let mut i = 0;
    while i < obligations.len() {
        if !redeemed(obligations[i].0, work, states) {
            return false;
        }
        i += 1;
    }
    true
}

const fn redeemed(slug: &str, work: &[&WorkItem], states: &[(WorkRef, WorkState)]) -> bool {
    let mut j = 0;
    while j < work.len() {
        if let WorkOrigin::Obligation(reference) = work[j].origin {
            if slug_eq(slug, reference.as_str()) {
                let mut k = 0;
                while k < states.len() {
                    if slug_eq(work[j].id, states[k].0.as_str()) {
                        return matches!(
                            states[k].1,
                            WorkState::Landed | WorkState::LandedFromHistory
                        );
                    }
                    k += 1;
                }
                return false;
            }
        }
        j += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::{BlastRadius, NonEmptyStr, SliceRef, WorkRef};

    dacc_core::declare_taxonomy! { Subsystem => [Core] }

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

    /// Решение 35: обязательство погашено только приземлённой работой с
    /// происхождением `WorkOrigin::Obligation`.
    #[test]
    fn obligations_are_redeemed_only_by_a_landed_work() {
        let obligation = Obligation {
            id: "o-fix-x",
            title: NonEmptyStr::new("Перейти на …"),
            discharged_when: NonEmptyStr::new("когда …"),
        };
        let work = WorkItem {
            id: "w-001",
            title: NonEmptyStr::new("Погасить o-fix-x"),
            slice: SliceRef::__from_scan("s-003"),
            origin: WorkOrigin::Obligation(ObligationRef::__from_scan("o-fix-x")),
            taxon: Subsystem::Core,
            radius: BlastRadius::Local,
            outcome: NonEmptyStr::new("готово"),
        };

        let obligations = [("o-fix-x", &obligation)];
        let work = [&work];

        let landed = [(WorkRef::__from_scan("w-001"), WorkState::Landed)];
        assert!(obligations_redeemed(&obligations, &work, &landed));

        let planned = [(WorkRef::__from_scan("w-001"), WorkState::Planned)];
        assert!(!obligations_redeemed(&obligations, &work, &planned));
    }
}
