//! Фасадный крейт DACC (решение 7): публичная поверхность для потребителя —
//! одна зависимость `dacc` вместо набора `dacc-core`, `dacc-knowledge`,
//! `dacc-work`, `dacc-scan`, `dacc-access`, `dacc-journal`.
//!
//! Макросы реэкспортируются, но их `$crate` — крейт-источник, поэтому для
//! разметки реестра потребителю нужны и конкретные крейты как зависимости.
//! Фасад снимает налог на перечисление имён в `use`, а не на зависимости.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub use dacc_access::{export_graph, export_slice, publish_channel, render_site, Site};
pub use dacc_core::{
    axis, date, declare_channels, declare_taxonomy, nonempty, nonempty_str, taxon, AdrRef,
    AnchorId, BlastRadius, BreakingRef, Channel, GateRef, InvariantRef, NonEmpty, NonEmptyStr,
    ObligationRef, ReviewRef, RfcRef, Severity, SliceRef, SupersededRef, Taxon, ThrustRef, WorkRef,
};
pub use dacc_journal::{
    fold, parse_event, read_dir, Event, Evidence, GateVerdict, Journal, Kind, Stage, Subject,
    Violation,
};
pub use dacc_knowledge::{
    adr, invariant, review, rfc, ArchitectureDecision, Breaking, DocStatus, DomainSpecification,
    Enforced, Invariant, InvariantStatus, Review,
};
pub use dacc_scan::{
    check_inline_links, emit_anchor_refs, emit_anchors, emit_refs, extract_inline_links,
    scan_anchors, scan_decisions, scan_specs, AnchorMode, ScanError, ScannedAnchor, ScannedSlug,
};
pub use dacc_work::{
    obligation, obligations_redeemed, radius_within_slice, slice, thrust, toil_within_local_radius,
    work, InquiryOutcome, Obligation, Slice, Thrust, WorkItem, WorkOrigin, WorkState,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Фасад переэкспортирует имена слоёв: один `use dacc::…` вместо набора.
    #[test]
    fn facade_reexports_the_public_surface() {
        let title = NonEmptyStr::new("заголовок");
        assert_eq!(title.as_str(), "заголовок");
        assert_eq!(DocStatus::Active, DocStatus::Active);
        assert_eq!(Breaking::No, Breaking::No);
        let toil = WorkOrigin::Toil {
            justification: NonEmptyStr::new("рутина"),
        };
        assert!(toil.lands_code());
    }
}
