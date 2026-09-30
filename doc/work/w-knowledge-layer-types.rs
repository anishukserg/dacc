use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-knowledge-layer-types",
    title: NonEmptyStr::new("Типы invariant! и review! — в слое знания"),
    slice: crate::slice::s_equivalents,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_knowledge_layer_boundary,
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "invariant! и review! живут в dacc-knowledge; скан порождает их реестр путём dacc_knowledge::Invariant / Review; ADR-045/046 называют слой знания."
    ),
);
