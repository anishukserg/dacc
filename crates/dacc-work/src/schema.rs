//! Схема слоя работы. Ссылки — пути к константам, порождённым сканом
//! реестров; правило, выразимое типом, валидатором не проверяется.

use dacc_core::{
    axis::Subsystem, AdrRef, BlastRadius, LimitationRef, NonEmpty, NonEmptyStr, ObligationRef,
    RfcRef, SliceRef, SupersededRef, Taxon, ThrustRef, WorkRef,
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
    /// Критерии решения — обязательные и непустые по типу: вопрос не погашается
    /// «когда-нибудь» без названной меры, по которой он считается решённым.
    pub criteria: NonEmpty<NonEmptyStr>,
}

/// Ограничение: расхождение фактического с заявленным, объявленное отдельной
/// записью реестра. Работа с происхождением [`WorkOrigin::Divergence`]
/// ссылается на него, а не называет расхождение свободным текстом, поэтому
/// необъявленное расхождение не компилируется.
#[derive(Debug)]
pub struct Limitation {
    pub id: &'static str,
    pub title: NonEmptyStr,
    /// Что расходится с заявленным — обязательное: без него запись не
    /// компилируется.
    pub violated: NonEmptyStr,
}

/// Инструкция повышения версии: шаг, который потребитель обязан сделать при
/// переходе с `from` на `to` (решение 44). Все четыре текстовых поля обязательны
/// и непусты по типу: шаг без способа исправления невыразим, а запись без
/// названия версий не связывает изменение с выпуском.
#[derive(Debug)]
pub struct Upgrade {
    pub id: &'static str,
    /// Версия до изменения.
    pub from: NonEmptyStr,
    /// Версия с изменением.
    pub to: NonEmptyStr,
    /// Что изменилось.
    pub subject: NonEmptyStr,
    /// Что потребитель обязан сделать.
    pub how: NonEmptyStr,
}

/// Происхождение работы: тип задачи и её обоснование — одно поле.
///
/// Перечисление неполное относительно части III: `Migration` появится вместе
/// с реестром, на который ссылается. `Mandate` и `DebtService` той же части
/// реализованы как запись [`Obligation`] и вариант [`WorkOrigin::Obligation`]
/// (решение 35). `#[non_exhaustive]` не ставится намеренно: новый вариант
/// обязан сломать сборку валидатора, а не молча остаться без правила.
/// Точка разметки кода для контракта «работа имеет основание»: ссылки из
/// записей инвариантов ведут сюда константой, порождённой сканом.
#[dacc_derive::doc_anchor(id = "work-origin")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkOrigin {
    /// Реализация принятого решения.
    Decision(AdrRef),
    /// Реализация доменной спецификации.
    Specification(RfcRef),
    /// Расхождение фактического с заявленным. Нарушенное утверждение живёт в
    /// записи [`Limitation`], а работа ссылается на него путём — необъявленное
    /// расхождение не компилируется.
    Divergence {
        specification: RfcRef,
        limitation: LimitationRef,
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

/// Работа с происхождением `Toil` не может объявить радиус выше `Local`:
/// рутина не трогает публичный API крейта. Функция `const`: скан порождает
/// утверждение на каждую единицу работы, и нарушение становится ошибкой
/// вычисления константы, а не находкой валидатора.
pub const fn toil_within_local_radius(work: &WorkItem) -> bool {
    if let WorkOrigin::Toil { .. } = work.origin {
        return matches!(work.radius, BlastRadius::Local);
    }
    true
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
    obligations: &[&Obligation],
    work: &[&WorkItem],
    states: &[(WorkRef, WorkState)],
) -> bool {
    let mut i = 0;
    while i < obligations.len() {
        if !redeemed(obligations[i].id, work, states) {
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

/// Каждое ограничение объявлено работой с происхождением
/// [`WorkOrigin::Divergence`], ссылающейся на него путём. Функция `const`: скан
/// порождает утверждение, и необъявленное ограничение — ошибка вычисления
/// константы, а не находка валидатора.
pub const fn limitations_declared(limitations: &[&Limitation], work: &[&WorkItem]) -> bool {
    let mut i = 0;
    while i < limitations.len() {
        if !declared(limitations[i].id, work) {
            return false;
        }
        i += 1;
    }
    true
}

const fn declared(slug: &str, work: &[&WorkItem]) -> bool {
    let mut j = 0;
    while j < work.len() {
        if let WorkOrigin::Divergence { limitation, .. } = work[j].origin {
            if slug_eq(slug, limitation.as_str()) {
                return true;
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
            criteria: dacc_core::nonempty_str!["названа мера решения"],
        };
        assert_eq!(obligation.id, "o-fix-x");
        assert_eq!(obligation.discharged_when.as_str(), "когда …");
        assert_eq!(
            obligation
                .criteria
                .iter()
                .map(NonEmptyStr::as_str)
                .collect::<Vec<_>>(),
            ["названа мера решения"]
        );

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
            criteria: dacc_core::nonempty_str!["названа мера решения"],
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

        let obligations = [&obligation];
        let work = [&work];

        let landed = [(WorkRef::__from_scan("w-001"), WorkState::Landed)];
        assert!(obligations_redeemed(&obligations, &work, &landed));

        let planned = [(WorkRef::__from_scan("w-001"), WorkState::Planned)];
        assert!(!obligations_redeemed(&obligations, &work, &planned));
    }

    /// Ограничение объявлено работой с происхождением Divergence, ссылающейся
    /// на него путём; работа иного происхождения ограничение не объявляет.
    #[test]
    fn limitations_are_declared_by_a_divergence_work() {
        let limitation = Limitation {
            id: "l-fix-x",
            title: NonEmptyStr::new("Ограничение"),
            violated: NonEmptyStr::new("фактическое расходится с заявленным"),
        };
        let declaring = WorkItem {
            id: "w-001",
            title: NonEmptyStr::new("Объявить ограничение"),
            slice: SliceRef::__from_scan("s-003"),
            origin: WorkOrigin::Divergence {
                specification: RfcRef::__from_scan("rfc-2026-002"),
                limitation: LimitationRef::__from_scan("l-fix-x"),
            },
            taxon: Subsystem::Core,
            radius: BlastRadius::Local,
            outcome: NonEmptyStr::new("готово"),
        };

        let limitations = [&limitation];
        assert!(limitations_declared(&limitations, &[&declaring]));

        let other = WorkItem {
            id: "w-002",
            title: NonEmptyStr::new("Другая работа"),
            slice: SliceRef::__from_scan("s-003"),
            origin: WorkOrigin::Toil {
                justification: NonEmptyStr::new("рутина"),
            },
            taxon: Subsystem::Core,
            radius: BlastRadius::Local,
            outcome: NonEmptyStr::new("готово"),
        };
        assert!(!limitations_declared(&limitations, &[&other]));
    }

    /// Рутина не может объявить радиус выше Local: правило выражает тип, а
    /// проверяет его компилятор. Не-Toil происхождение радиус не ограничивает.
    #[test]
    fn toil_is_capped_at_local_radius() {
        let toil = |radius| WorkItem {
            id: "w-001",
            title: NonEmptyStr::new("рутина"),
            slice: SliceRef::__from_scan("s-003"),
            origin: WorkOrigin::Toil {
                justification: NonEmptyStr::new("рутина"),
            },
            taxon: Subsystem::Core,
            radius,
            outcome: NonEmptyStr::new("готово"),
        };
        assert!(toil_within_local_radius(&toil(BlastRadius::Local)));
        assert!(!toil_within_local_radius(&toil(BlastRadius::Crate)));

        let decision = WorkItem {
            id: "w-002",
            title: NonEmptyStr::new("решение"),
            slice: SliceRef::__from_scan("s-003"),
            origin: WorkOrigin::Decision(AdrRef::__from_scan("adr-2026-001")),
            taxon: Subsystem::Core,
            radius: BlastRadius::Crate,
            outcome: NonEmptyStr::new("готово"),
        };
        assert!(toil_within_local_radius(&decision));
    }
}
