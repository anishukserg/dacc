use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(61,
    title: NonEmptyStr::new("Явное дерево калитки уважает .gitignore"),
    slice: crate::slice::s0021,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Рабочее дерево калитка обходит по списку git и .gitignore уважает, а явное дерево (gate --repo <root> <dir>) обходит вручную, пропуская только target/ и .git/: игнорируемые файлы в нём попадают в шаги внешних имён и ссылок (решение 8)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Обход подкаталога репозитория берёт тот же список git ls-files --cached --others --exclude-standard, ограниченный каталогом; сценарий показывает, что файл в игнорируемом каталоге явного дерева калитка не видит, а тот же текст в неигнорируемом файле — видит."
    ),
);
