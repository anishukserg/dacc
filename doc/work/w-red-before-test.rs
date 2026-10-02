use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-red-before-test",
    title: NonEmptyStr::new("RedBefore доказывает тест, а не имя функции"),
    slice: crate::slice::s_proof_close_honesty,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_red_before_name },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("RedBefore принимает имя, точно совпадающее с функцией теста дерева: fn <имя>( и атрибут с test над ней; префикс имени и функция без атрибута отказываются кодом red-before-unknown; сценарий падал до починки"),
);
