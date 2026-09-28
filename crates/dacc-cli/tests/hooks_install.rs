//! Сценарии установки хуков (работа 50). Установка ставит все правила или не
//! ставит ни одного: частичная установка выглядит успешной, а пропущенный
//! commit-msg означает, что основание коммита не проверяется вовсе.

mod common;

use common::TempRepo;
use std::fs;

const FOREIGN: &str = "#!/bin/sh\n# проверка проекта\nexec cargo xtask commit-msg \"$1\"\n";

/// Хук проекта в каталоге хуков DACC: каталог и имена файлов совпадают.
fn with_foreign_hook(repo: &TempRepo, name: &str) {
    repo.write(&format!(".githooks/{name}"), FOREIGN);
}

#[test]
fn a_foreign_hook_stops_the_whole_installation() {
    let repo = TempRepo::new("hooks-foreign");
    with_foreign_hook(&repo, "commit-msg");

    let run = repo.tool(&["hooks", "install"]);
    assert_ne!(run.code, 0, "{}", run.output());
    assert!(
        run.output().contains("commit-msg"),
        "отказ не называет файл: {}",
        run.output()
    );
    // Подсказка ведёт к делегированию: проверка темы проекта задаётся
    // настройкой, а основание остаётся за DACC (решение 20).
    assert!(
        run.output().contains("message_command"),
        "отказ не подсказывает делегирование: {}",
        run.output()
    );

    // Ни одного файла до отказа: частичная установка запрещена.
    for name in ["pre-commit", "pre-push"] {
        assert!(
            !repo.path(&format!(".githooks/{name}")).exists(),
            "{name} записан несмотря на отказ: {}",
            run.output()
        );
    }
    assert_eq!(
        fs::read_to_string(repo.path(".githooks/commit-msg")).expect("чужой хук на месте"),
        FOREIGN,
        "чужой хук изменён"
    );
    assert!(
        repo.git_code(&["config", "--get", "core.hooksPath"]) != 0,
        "путь хуков выставлен при отказе"
    );
}

#[test]
fn our_own_hook_is_rewritten_and_the_path_is_set() {
    let repo = TempRepo::new("hooks-ours");

    let run = repo.tool(&["hooks", "install"]);
    assert_eq!(run.code, 0, "{}", run.output());
    for name in ["pre-commit", "commit-msg", "pre-push"] {
        assert!(
            repo.path(&format!(".githooks/{name}")).exists(),
            "{name} не записан: {}",
            run.output()
        );
    }
    assert_eq!(
        repo.git(&["config", "--get", "core.hooksPath"]).trim(),
        ".githooks"
    );

    // Хук прежней версии — наш: он переписывается, а не оставляется как есть,
    // иначе обновление инструмента становится ручной работой.
    let old = "#!/bin/sh\n# DACC rules (decision 14): the hook calls the installed cargo dacc.\nexec cargo dacc hook pre-commit\n";
    fs::write(repo.path(".githooks/pre-commit"), old).expect("хук перезаписан");
    let run = repo.tool(&["hooks", "install"]);
    assert_eq!(run.code, 0, "{}", run.output());
    let text = fs::read_to_string(repo.path(".githooks/pre-commit")).expect("хук читается");
    assert!(
        text.contains("RUSTUP_AUTO_INSTALL=0"),
        "прежняя версия хука не обновлена: {text}"
    );
}

#[test]
fn a_foreign_hook_is_replaced_only_on_demand() {
    let repo = TempRepo::new("hooks-force");
    with_foreign_hook(&repo, "commit-msg");

    let run = repo.tool(&["hooks", "install", "--force"]);
    assert_eq!(run.code, 0, "{}", run.output());
    let text = fs::read_to_string(repo.path(".githooks/commit-msg")).expect("хук читается");
    assert!(
        text.contains("cargo dacc hook commit-msg"),
        "чужой хук не заменён: {text}"
    );
}
