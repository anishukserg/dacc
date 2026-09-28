use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(57,
    title: NonEmptyStr::new("Обход дерева в калитке уважает .gitignore"),
    slice: crate::slice::s0019,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Калитка обходит дерево, пропуская только каталоги target/ и .git/: игнорируемые .gitignore пути (.venv-docs с site-packages) и чужой битый license.txt ломают шаги внешних имён и ссылок, хотя в репозиторий не входят (решение 8)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Обход рабочего дерева перечисляет файлы, которые git считает частью проекта (отслеживаемые и неигнорируемые неотслеживаемые); сценарий показывает, что битый файл в игнорируемом каталоге не останавливает калитку, а в отслеживаемом — останавливает."
    ),
);
