//! Публикация канала (решение 21): фильтр документов по каналу и отказ ссылки
//! через границу канала.

use dacc_core::Channel;
use dacc_knowledge::{ArchitectureDecision, DomainSpecification};
use std::collections::HashMap;

/// Документы канала: решения и спецификации этого канала.
pub type ChannelDocuments<'a> = (
    Vec<(&'a str, &'a ArchitectureDecision)>,
    Vec<(&'a str, &'a DomainSpecification)>,
);

/// Документы канала: решения и спецификации только этого канала, либо ошибка
/// при ссылке из документа канала в документ другого канала (решение 21).
pub fn publish_channel<'a>(
    decisions: &'a [(&'a str, &'a ArchitectureDecision)],
    specs: &'a [(&'a str, &'a DomainSpecification)],
    channel: Channel,
) -> Result<ChannelDocuments<'a>, String> {
    let spec_channel: HashMap<&str, Channel> =
        specs.iter().map(|(id, s)| (*id, s.channel)).collect();
    let decision_channel: HashMap<&str, Channel> =
        decisions.iter().map(|(id, d)| (*id, d.channel)).collect();

    let decisions: Vec<_> = decisions
        .iter()
        .copied()
        .filter(|(_, d)| d.channel == channel)
        .collect();
    let specs: Vec<_> = specs
        .iter()
        .copied()
        .filter(|(_, s)| s.channel == channel)
        .collect();

    for (id, d) in &decisions {
        for rfc in d.related_rfcs {
            if let Some(other) = spec_channel.get(rfc.as_str()) {
                if *other != channel {
                    return Err(format!(
                        "decision {id} links to rfc {} of channel {} — cross-channel link",
                        rfc.as_str(),
                        other.as_str()
                    ));
                }
            }
        }
    }
    for (id, s) in &specs {
        for adr in s.decided_by {
            if let Some(other) = decision_channel.get(adr.as_str()) {
                if *other != channel {
                    return Err(format!(
                        "spec {id} is decided by adr {} of channel {} — cross-channel link",
                        adr.as_str(),
                        other.as_str()
                    ));
                }
            }
        }
    }

    Ok((decisions, specs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::{nonempty_str, AdrRef, RfcRef};
    use dacc_knowledge::{Breaking, DocStatus};

    dacc_core::declare_channels! { Public, Internal }

    fn decision(channel: Channel, rfc: &'static [RfcRef]) -> ArchitectureDecision {
        ArchitectureDecision {
            title: "Решение",
            status: DocStatus::Active,
            subsystems: &[],
            channel,
            context: "Контекст.",
            decision: "Решено.",
            trade_offs: &[],
            constraints: &[],
            authors: nonempty_str!["Автор"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 29).unwrap(),
            breaking: Breaking::No,
            code_refs: &[],
            related_rfcs: rfc,
        }
    }

    fn spec(channel: Channel, decided_by: &'static [AdrRef]) -> DomainSpecification {
        DomainSpecification {
            title: "Спецификация",
            status: DocStatus::Draft,
            target: &[],
            channel,
            goal: "Цель.",
            input_contract: "Вход.",
            output_contract: "Выход.",
            invariants: nonempty_str!["инвариант"],
            authors: nonempty_str!["Автор"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 29).unwrap(),
            decided_by,
        }
    }

    #[test]
    fn filters_by_channel() {
        let pub_d = decision(channel::Public, &[]);
        let int_d = decision(channel::Internal, &[]);
        let pub_s = spec(channel::Public, &[]);
        let int_s = spec(channel::Internal, &[]);
        let decisions = [("a", &pub_d), ("b", &int_d)];
        let specs = [("r1", &pub_s), ("r2", &int_s)];

        let (ds, ss) = publish_channel(&decisions, &specs, channel::Public).unwrap();
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].0, "a");
        assert_eq!(ss.len(), 1);
        assert_eq!(ss[0].0, "r1");
    }

    #[test]
    fn rejects_a_cross_channel_link() {
        const RFC_INTERNAL: RfcRef = RfcRef::__from_scan("r-internal");
        let pub_d = decision(channel::Public, &[RFC_INTERNAL]);
        let int_s = spec(channel::Internal, &[]);
        let decisions = [("a", &pub_d)];
        let specs = [("r-internal", &int_s)];

        let err = publish_channel(&decisions, &specs, channel::Public).unwrap_err();
        assert!(err.contains("cross-channel link"), "{err}");
    }
}
