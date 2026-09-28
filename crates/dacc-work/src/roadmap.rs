//! Дорожная карта — первая проекция реестра (решение 21): Markdown из
//! скомпилированного реестра, а не из повторного разбора. Проекция не может
//! разойтись с кодом, потому что данные берутся из констант, уже проверенных
//! компилятором.

use crate::{Slice, Thrust, WorkItem, WorkState};
use dacc_core::{SliceRef, WorkRef};
use std::fmt::Write as _;

/// Рендерит направления, их срезы с признаком закрытия и работы с состояниями
/// журнала одним документом Markdown. `commit` — коммит, из которого собран
/// реестр; пустая строка отображается как неизвестный.
pub fn render_roadmap(
    thrusts: &[&Thrust],
    slices: &[&Slice],
    work: &[&WorkItem],
    states: &[(WorkRef, WorkState)],
    closed: &[SliceRef],
    commit: &str,
) -> String {
    let mut out = String::from("# Roadmap\n\n");
    if commit.is_empty() {
        out.push_str("Built from an unknown commit.\n");
    } else {
        let _ = writeln!(out, "Built from commit `{commit}`.");
    }

    let mut thrusts: Vec<&&Thrust> = thrusts.iter().collect();
    thrusts.sort_by_key(|t| t.id);
    for thrust in thrusts {
        let _ = writeln!(
            out,
            "\n## {} — {}\n",
            id_str('t', thrust.id),
            thrust.title.as_str()
        );
        let _ = writeln!(out, "{}\n", thrust.outcome.as_str());

        let mut own: Vec<&&Slice> = slices
            .iter()
            .filter(|s| s.thrust.index() == thrust.id)
            .collect();
        own.sort_by_key(|s| s.id);
        for slice in own {
            let suffix = if closed.iter().any(|c| c.index() == slice.id) {
                " — closed"
            } else {
                ""
            };
            let _ = writeln!(
                out,
                "\n### {} — {}{}\n",
                id_str('s', slice.id),
                slice.title.as_str(),
                suffix
            );
            let _ = writeln!(out, "{}\n", slice.outcome.as_str());

            let mut items: Vec<&&WorkItem> = work
                .iter()
                .filter(|w| w.slice.index() == slice.id)
                .collect();
            items.sort_by_key(|w| w.id);
            let has_items = !items.is_empty();
            for item in items {
                let state = states
                    .iter()
                    .find(|(r, _)| r.index() == item.id)
                    .map_or(WorkState::Planned, |(_, s)| *s);
                let _ = writeln!(
                    out,
                    "- {} {} — {}",
                    id_str('w', item.id),
                    item.title.as_str(),
                    state_name(state)
                );
            }
            if has_items {
                out.push('\n');
            }
        }
    }
    out
}

fn id_str(prefix: char, id: u32) -> String {
    format!("{prefix}{id:04}")
}

fn state_name(state: WorkState) -> &'static str {
    match state {
        WorkState::Planned => "planned",
        WorkState::Started => "started",
        WorkState::Landed | WorkState::LandedFromHistory => "landed",
        WorkState::Abandoned => "abandoned",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Slice, Thrust, WorkItem, WorkOrigin};
    use dacc_core::{BlastRadius, NonEmptyStr, RfcRef, SliceRef, ThrustRef, WorkRef};

    dacc_core::declare_taxonomy! { Subsystem => [Core] }

    #[test]
    fn renders_thrusts_slices_and_work_states() {
        let thrust = Thrust {
            id: 1,
            title: NonEmptyStr::new("Направление"),
            outcome: NonEmptyStr::new("Исход направления"),
        };
        let slice = Slice {
            id: 3,
            title: NonEmptyStr::new("Срез"),
            thrust: ThrustRef::__from_scan(1),
            outcome: NonEmptyStr::new("Исход среза"),
            specification: RfcRef::__from_scan("rfc-2026-001"),
            max_radius: BlastRadius::Crate,
        };
        let work = WorkItem {
            id: 7,
            title: NonEmptyStr::new("Работа"),
            slice: SliceRef::__from_scan(3),
            origin: WorkOrigin::Toil {
                justification: NonEmptyStr::new("рутина"),
            },
            taxon: Subsystem::Core,
            radius: BlastRadius::Local,
            outcome: NonEmptyStr::new("готово"),
        };
        let states = [(WorkRef::__from_scan(7), WorkState::Started)];
        let closed = [SliceRef::__from_scan(3)];

        let out = render_roadmap(&[&thrust], &[&slice], &[&work], &states, &closed, "abc123");
        assert!(out.contains("Built from commit `abc123`."), "{out}");
        assert!(out.contains("## t0001 — Направление"), "{out}");
        assert!(out.contains("### s0003 — Срез — closed"), "{out}");
        assert!(out.contains("- w0007 Работа — started"), "{out}");
    }
}
