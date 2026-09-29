//! Свёртка журнала: стадия каждой работы, закрытые срезы и нарушения автомата.
//!
//! События упорядочиваются по времени; при равном времени — по порядку
//! автомата (начало, проверка, завершение, закрытие среза), затем по имени
//! файла. Нарушающее событие не меняет стадию: следующее событие проверяется
//! против последнего законного.

use crate::event::{Event, Evidence, Kind, Subject};
use std::collections::BTreeMap;

/// Стадия работы после свёртки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
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

impl Stage {
    /// После этой стадии событий у работы нет.
    pub fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Landed | Self::LandedFromHistory | Self::Abandoned
        )
    }
}

/// Результат свёртки.
#[derive(Debug, Default)]
pub struct Journal {
    /// Стадия каждой работы, у которой есть законные события.
    pub works: BTreeMap<String, Stage>,
    /// Закрытые срезы и файл закрывшего события.
    pub closed_slices: BTreeMap<String, String>,
    /// Последнее законное событие каждой работы.
    pub last_event: BTreeMap<String, String>,
    /// Деревья из событий gate каждой работы — доказательства.
    pub proofs: BTreeMap<String, Vec<String>>,
    /// Стадия каждого обязательства (решение 35): `Landed`, если приземлена
    /// работа с происхождением `WorkOrigin::Obligation`, иначе `Planned`.
    pub obligations: BTreeMap<String, Stage>,
}

impl Journal {
    /// Стадия работы; работа без событий запланирована.
    pub fn stage(&self, work: &str) -> Stage {
        self.works.get(work).copied().unwrap_or(Stage::Planned)
    }
}

/// Нарушение: файл события и причина.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub file: String,
    pub reason: String,
}

/// Сворачивает события в состояние и перечисляет нарушения автомата.
///
/// `obligations` — slug обязательств реестра, `discharges` — пары
/// (slug работы, slug обязательства) для работ с происхождением
/// `WorkOrigin::Obligation`. Обязательство погашено, когда такая работа
/// приземлена (решение 35).
pub fn fold(
    events: &[Event],
    obligations: &[&str],
    discharges: &[(&str, &str)],
) -> (Journal, Vec<Violation>) {
    let mut ordered: Vec<&Event> = events.iter().collect();
    ordered.sort_by(|a, b| {
        (a.at.as_str(), rank(&a.kind), a.file.as_str()).cmp(&(
            b.at.as_str(),
            rank(&b.kind),
            b.file.as_str(),
        ))
    });
    let mut journal = Journal::default();
    let mut violations = Vec::new();
    for event in ordered {
        let outcome = match &event.subject {
            Subject::Slice(slice) => close_slice(&mut journal, slice, event),
            Subject::Work(work) => advance_work(&mut journal, work, event),
            Subject::Obligation(o) => Err(format!(
                "obligation {o} has no journal events: redemption is a landed work"
            )),
        };
        if let Err(reason) = outcome {
            violations.push(Violation {
                file: event.file.clone(),
                reason,
            });
        }
    }

    for obligation in obligations {
        let discharged = discharges.iter().any(|(work, obl)| {
            obl == obligation
                && matches!(
                    journal.stage(work),
                    Stage::Landed | Stage::LandedFromHistory
                )
        });
        journal.obligations.insert(
            (*obligation).to_owned(),
            if discharged {
                Stage::Landed
            } else {
                Stage::Planned
            },
        );
    }

    (journal, violations)
}

/// Порядок вида события внутри одной секунды — порядок автомата. Иначе события
/// одной секунды упорядочились бы по имени файла, и проверка шла бы раньше
/// начала.
fn rank(kind: &Kind) -> u8 {
    match kind {
        Kind::Started => 0,
        Kind::Gate { .. } => 1,
        Kind::Landed { .. } | Kind::Abandoned { .. } => 2,
        Kind::Closed => 3,
    }
}

fn close_slice(journal: &mut Journal, slice: &str, event: &Event) -> Result<(), String> {
    if let Some(previous) = journal.closed_slices.get(slice) {
        return Err(format!(
            "slice {} is already closed by event {previous}",
            event.subject.id()
        ));
    }
    journal
        .closed_slices
        .insert(slice.to_owned(), event.file.clone());
    Ok(())
}

fn advance_work(journal: &mut Journal, work: &str, event: &Event) -> Result<(), String> {
    let id = event.subject.id();
    let stage = journal.stage(work);
    if stage.is_finished() {
        let last = journal.last_event.get(work).map_or("", String::as_str);
        return Err(format!("work {id} is already finished by event {last}"));
    }
    let next = match (&event.kind, stage) {
        (Kind::Started, Stage::Planned) => Stage::Started,
        (Kind::Started, _) => {
            let last = journal.last_event.get(work).map_or("", String::as_str);
            return Err(format!("work {id} is already started by event {last}"));
        }
        (Kind::Gate { tree, .. }, Stage::Started) => {
            journal
                .proofs
                .entry(work.to_owned())
                .or_default()
                .push(tree.clone());
            Stage::Started
        }
        (Kind::Gate { .. }, _) => {
            return Err(format!("gate on work {id} before its start"));
        }
        (
            Kind::Landed {
                evidence: Evidence::Gate,
                tree,
                ..
            },
            Stage::Started,
        ) => {
            let proven = journal
                .proofs
                .get(work)
                .is_some_and(|trees| trees.contains(tree));
            if !proven {
                return Err(format!(
                    "landing of work {id} without proof: no gate event with tree {tree}"
                ));
            }
            Stage::Landed
        }
        (
            Kind::Landed {
                evidence: Evidence::Gate,
                ..
            },
            _,
        ) => return Err(format!("landing of work {id} without its start")),
        (
            Kind::Landed {
                evidence: Evidence::History,
                ..
            },
            Stage::Planned,
        ) => Stage::LandedFromHistory,
        (
            Kind::Landed {
                evidence: Evidence::History,
                ..
            },
            _,
        ) => {
            return Err(format!(
                "work {id} is already in the journal: history is only for work without events"
            ));
        }
        (Kind::Abandoned { .. }, _) => Stage::Abandoned,
        (Kind::Closed, _) => {
            return Err("a closed event belongs to a slice, not to work".to_owned())
        }
    };
    journal.works.insert(work.to_owned(), next);
    journal
        .last_event
        .insert(work.to_owned(), event.file.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::GateVerdict;

    const TREE_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TREE_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn at(second: u32) -> String {
        format!("2026-09-11T03:15:{second:02}Z")
    }

    fn work(second: u32, kind: Kind) -> Event {
        Event::new(Subject::Work("w-022".to_owned()), at(second), kind)
    }

    fn gate(tree: &str) -> Kind {
        Kind::Gate {
            gate: "commit".into(),
            tree: tree.into(),
            verdict: GateVerdict::Prose("GATE OK".into()),
        }
    }

    fn landed(tree: &str, evidence: Evidence) -> Kind {
        Kind::Landed {
            commit: TREE_A.into(),
            tree: tree.into(),
            evidence,
        }
    }

    fn reasons(events: &[Event]) -> Vec<String> {
        fold(events, &[], &[])
            .1
            .into_iter()
            .map(|v| v.reason)
            .collect()
    }

    #[test]
    fn started_gate_landed_on_the_same_tree_is_legal() {
        let events = [
            work(1, Kind::Started),
            work(2, gate(TREE_A)),
            work(3, landed(TREE_A, Evidence::Gate)),
        ];
        let (journal, violations) = fold(&events, &[], &[]);
        assert!(violations.is_empty(), "{violations:?}");
        assert_eq!(journal.stage("w-022"), Stage::Landed);
        assert_eq!(journal.stage("w-023"), Stage::Planned);
    }

    #[test]
    fn order_comes_from_time_not_from_the_input() {
        let events = [
            work(3, landed(TREE_A, Evidence::Gate)),
            work(1, Kind::Started),
            work(2, gate(TREE_A)),
        ];
        assert!(fold(&events, &[], &[]).1.is_empty());
    }

    #[test]
    fn illegal_transitions_name_their_reason() {
        let cases: [(&[Event], &str); 6] = [
            (&[work(1, gate(TREE_A))], "before its start"),
            (
                &[work(1, landed(TREE_A, Evidence::Gate))],
                "without its start",
            ),
            (
                &[
                    work(1, Kind::Started),
                    work(2, gate(TREE_B)),
                    work(3, landed(TREE_A, Evidence::Gate)),
                ],
                "without proof",
            ),
            (
                &[work(1, Kind::Started), work(2, Kind::Started)],
                "is already started",
            ),
            (
                &[
                    work(1, Kind::Started),
                    work(2, landed(TREE_A, Evidence::History)),
                ],
                "only for work without events",
            ),
            (
                &[
                    work(1, Kind::Abandoned { reason: "x".into() }),
                    work(2, Kind::Started),
                ],
                "is already finished",
            ),
        ];
        for (events, reason) in cases {
            let found = reasons(events);
            assert!(
                found.iter().any(|r| r.contains(reason)),
                "ожидалось «{reason}», получено {found:?}"
            );
        }
    }

    #[test]
    fn history_lands_a_planned_work_and_slices_close_once() {
        let close =
            |second| Event::new(Subject::Slice("s-007".to_owned()), at(second), Kind::Closed);
        let events = [
            work(1, landed(TREE_A, Evidence::History)),
            close(2),
            close(3),
        ];
        let (journal, violations) = fold(&events, &[], &[]);
        assert_eq!(journal.stage("w-022"), Stage::LandedFromHistory);
        assert_eq!(journal.closed_slices.len(), 1);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].reason.contains("is already closed"),
            "{violations:?}"
        );
    }

    /// Решение 35: обязательство погашено приземлённой работой, иначе остаётся
    /// запланированным; событие обязательства — нарушение.
    #[test]
    fn obligations_are_redeemed_by_a_landed_discharge_work() {
        let events = [
            work(1, Kind::Started),
            work(2, gate(TREE_A)),
            work(3, landed(TREE_A, Evidence::Gate)),
        ];
        let (journal, violations) = fold(&events, &["o-fix-x", "o-open"], &[("w-022", "o-fix-x")]);
        assert!(violations.is_empty(), "{violations:?}");
        assert_eq!(journal.obligations.get("o-fix-x"), Some(&Stage::Landed));
        assert_eq!(journal.obligations.get("o-open"), Some(&Stage::Planned));

        let obligation_event = Event::new(
            Subject::Obligation("o-fix-x".to_owned()),
            at(1),
            Kind::Closed,
        );
        let (_, violations) = fold(&[obligation_event], &[], &[]);
        assert!(
            violations
                .iter()
                .any(|v| v.reason.contains("no journal events")),
            "{violations:?}"
        );
    }
}
