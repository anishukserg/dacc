//! Сверка версии инструмента с деревом (ADR-2026-048, работа
//! w-tool-version-check).
//!
//! Мера дерева — версия dacc-work в Cargo.lock реестра: она объявляет схему
//! записей и журнала, которую сворачивает build.rs. Версия бинарника — его
//! CARGO_PKG_VERSION. Расхождение в обе стороны — отказ мутирующих команд и
//! калитки с кодом tool-version-drift; дерево без Cargo.lock сверки не
//! проходит и не отказывает.

use std::fs;
use std::path::Path;

/// Текст отказа о расхождении версий; `None` — сверка сошлась или меры нет.
pub(crate) fn drift(root: &Path) -> Option<String> {
    let tree = tree_version(root)?;
    let tool = env!("CARGO_PKG_VERSION");
    (tree != tool).then(|| {
        format!(
            "the tool runs {tool}, the tree pins dacc-work {tree}: reinstall cargo-dacc or move the registry onto the tool's version"
        )
    })
}

/// Версия dacc-work из Cargo.lock дерева; `None` — меры нет.
fn tree_version(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("Cargo.lock")).ok()?;
    let mut in_package = false;
    let mut is_dacc_work = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            in_package = true;
            is_dacc_work = false;
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed == "name = \"dacc-work\"" {
            is_dacc_work = true;
            continue;
        }
        if is_dacc_work {
            if let Some(version) = trimmed.strip_prefix("version = \"") {
                return Some(version.trim_end_matches('"').to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo_with_lock(name: &str, text: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir()
            .join("dacc-cli")
            .join(format!("version-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Cargo.lock"), text).unwrap();
        root
    }

    /// Версия читается из записи dacc-work, а не из первой попавшейся.
    #[test]
    fn tree_version_reads_the_dacc_work_entry() {
        let root = repo_with_lock(
            "reads",
            "version = 4\n\n[[package]]\nname = \"aaa\"\nversion = \"9.9.9\"\n\n[[package]]\nname = \"dacc-work\"\nversion = \"1.2.3\"\n",
        );
        assert_eq!(tree_version(&root), Some("1.2.3".to_owned()));
        assert!(drift(&root).is_some());
    }

    /// Дерево без lock сверки не проходит: меры нет и отказа нет.
    #[test]
    fn a_tree_without_the_measure_does_not_drift() {
        let root = repo_with_lock("absent", "");
        fs::remove_file(root.join("Cargo.lock")).unwrap();
        assert_eq!(tree_version(&root), None);
        assert!(drift(&root).is_none());
    }
}
