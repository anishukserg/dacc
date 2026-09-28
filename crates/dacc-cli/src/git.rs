//! Запуск git.

use crate::config::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Команда git в каталоге `dir`.
pub fn command(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir);
    cmd
}

/// Стандартный вывод git без завершающих переводов строки; `None`, если git не
/// запустился или завершился с ошибкой.
pub fn read(dir: &Path, args: &[&str]) -> Option<String> {
    let out = command(dir)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
}

/// Завершилась ли команда git успехом; вывод отбрасывается.
pub fn succeeds(dir: &Path, args: &[&str]) -> bool {
    command(dir)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Репозиторий: корень рабочего дерева, каталог git этой рабочей копии, общий
/// каталог git репозитория и настройка рабочего дерева (решение 20).
///
/// В связанной рабочей копии каталоги разные, и разделение содержательно.
/// Приватное принадлежит копии: индекс, блокировка коммита, выгрузка дерева.
/// Общее принадлежит репозиторию: доказательства калитки и список имён внешних
/// проектов. Иначе калитка, пройденная в изолированной сессии, переставала бы
/// быть доказательством для того же дерева (решения 4, 9 и 15).
pub struct Repo {
    pub root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub config: Config,
}

impl Repo {
    /// Репозиторий, которому принадлежит каталог `dir`, с настройкой из его
    /// рабочего дерева. `Err` — не репозиторий или негодная настройка: по
    /// испорченной настройке инструмент не работает.
    ///
    /// Общий каталог берётся в абсолютной форме; у git без такой формы он
    /// читается относительным и раскрывается от `dir`.
    pub fn discover(dir: &Path) -> Result<Repo, String> {
        let root = read(dir, &["rev-parse", "--show-toplevel"])
            .ok_or_else(|| "not a git repository".to_owned())?;
        let git_dir = read(dir, &["rev-parse", "--absolute-git-dir"])
            .ok_or_else(|| "not a git repository".to_owned())?;
        let common_dir = read(
            dir,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            read(dir, &["rev-parse", "--git-common-dir"])
                .map(|path| dir.join(path))
                .and_then(|path| fs::canonicalize(path).ok())
        })
        .unwrap_or_else(|| PathBuf::from(&git_dir));
        let root = PathBuf::from(root);
        let config = Config::read_dir(&root)?;
        Ok(Repo {
            root,
            git_dir: PathBuf::from(git_dir),
            common_dir,
            config,
        })
    }
}
