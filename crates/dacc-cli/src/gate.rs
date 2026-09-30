//! Калитка коммита по решениям 8, 12, 13 и 15: один набор шагов и машинный
//! вердикт.
//!
//! ```text
//! cargo dacc gate [--repo <directory>] [--journal-only] [--full] [--format json|text] [<tree>]
//! ```
//!
//! Хук pre-commit передаёт выгруженное дерево коммита; без аргумента
//! проверяется рабочее дерево. Сборка идёт в `target/gate*` корня репозитория,
//! чтобы кэш переживал выгрузки; cargo запускается из самого дерева, чтобы
//! тулчейн брался из его `rust-toolchain.toml`.
//!
//! Настройку (`dacc.toml`) калитка читает из проверяемого дерева, как
//! `Cargo.toml` и `deny.toml`: оттуда берутся каталог журнала, манифест крейта
//! документов, пол числа прошедших атакующих doctest и команда проекта
//! (решение 20).
//!
//! Шаги, от дешёвых к дорогим:
//!
//! 1. внешние имена — по локальному списку в каталоге git (решение 9); без
//!    списка шаг называется невыполненным, а не пройденным;
//! 2. относительные ссылки в markdown ведут в существующие файлы;
//! 3. журнал: файлы событий из HEAD не изменены и не удалены, приземления
//!    совпадают с историей git (решение 15);
//! 4. команда проекта из `gate_command`, если она задана: заменяет шаги 5–7,
//!    10, 11 и 13 (решение 28). Её отказ — отказ калитки;
//! 5. форматирование — `cargo fmt --check`;
//! 6. clippy без предупреждений на всех целях;
//! 7. сборка всех целей с константными проверками реестров и тесты;
//! 8. сверка кодов ошибок работает: пробная атака с неверным кодом падает, с
//!    верным — проходит (решение 12);
//! 9. атаки — doctest со сверкой кодов под `RUSTC_BOOTSTRAP=1`, в отдельном
//!    каталоге сборки, с полом по числу прошедших;
//! 10. документация без предупреждений и битых внутренних ссылок;
//! 11. сборка всех целей на минимальной версии из `rust-version`;
//! 12. проба сверки кодов и атаки на минимальной версии, с тем же полом;
//! 13. зависимости — политика `deny.toml` по сохранённой базе уязвимостей, без
//!     сети (решение 13).
//!
//! Команда проекта идёт до шагов cargo: дешёвые проверки DACC отказывают
//! первыми, а команда проекта обычно сама включает форматирование, clippy и
//! тесты, поэтому шаги 5–7, 10, 11 и 13 при заданной команде пропускаются, а
//! проба сверки кодов и атаки (шаги 8, 9, 12) остаются — это ядро DACC, его не
//! заменяет никакая команда проекта.
//!
//! С `--journal-only` — для коммита, дерево которого без журнала уже прошло
//! калитку, — выполняются шаги 1–3 и сборка крейта документов, где журнал
//! сворачивается; команда проекта не выполняется: это дерево уже проверено.
//!
//! Два яруса (решение 33): без `--full` исполняется ярус коммита — шаги 1–10;
//! `--full` добавляет полный ярус закрытия работы — шаги 11–13 (MSRV и
//! зависимости). Хук pre-commit исполняет ярус коммита, чтобы коммит был
//! дешёвым; полный ярус исполняет `work land` при приземлении работы, и его
//! доказательство несёт `msrv`. CI на пути в master — тоже полный ярус.
//!
//! Отсутствующий инструмент — отказ шага, а не пропуск. Шаг без предмета
//! проверки называется невыполненным, а не пройденным.
//!
//! Код возврата: 0 — пройдено; 1 — шаг упал; 2 — ошибка запуска. Последняя
//! строка — `GATE OK (<n> of <m>; …)` или `GATE FAIL: <step>`; с
//! `--format json` — объект с полями `ok`, `passed`, `total`, `attacks`, `msrv`,
//! `msrv_attacks`, `skipped`, либо `step` и `code` при отказе.

use crate::code::{self, Code};
use crate::config::Config;
use crate::format::{self, Format};
use crate::{git, layout, proof};
use dacc_journal::Kind;
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// Число собственных шагов полной калитки. Команда проекта заменяет шесть
/// стандартных шагов cargo и добавляет себя (решение 28).
const DACC_TOTAL: usize = 12;

/// Сколько стандартных шагов cargo заменяет команда проекта: форматирование,
/// clippy, тесты, документация, проверка MSRV, зависимости (решение 28).
const DELEGATED_STEPS: usize = 6;

/// Число шагов калитки журнала.
const JOURNAL_ONLY_TOTAL: usize = 4;

/// Сколько строк ошибок из журнала шага показывать при отказе.
const ERROR_LINES: usize = 40;

/// Сколько последних строк журнала шага показывать при отказе.
const TAIL_LINES: usize = 12;

/// Манифест пробы сверки кодов: собственное рабочее пространство, чтобы cargo
/// не искал родительское.
const PROBE_MANIFEST: &str = r#"[package]
name = "dacc-gate-probe"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]
"#;

/// Проба сверки кодов: неверный код обязан упасть, верный — пройти.
const PROBE_LIB: &str = r#"//! Probe for error code matching in attacks (decision 12).
//!
//! Wrong code: the body yields E0308 while E0080 is declared — must fail.
//!
//! ```compile_fail,E0080
//! let _: u32 = "not a number";
//! ```
//!
//! Right code: the same body — must pass.
//!
//! ```compile_fail,E0308
//! let _: u32 = "not a number";
//! ```
"#;

/// `cargo dacc gate`.
pub fn run(args: &[OsString]) -> u8 {
    run_with_verdict(args).0
}

/// Калитка с вердиктом: код возврата и последняя строка. Формат вывода — из
/// `--format`, по умолчанию человекочитаемый.
pub fn run_with_verdict(args: &[OsString]) -> (u8, String) {
    let format = format::scan(args);
    let outcome = Args::parse(args)
        .and_then(|args| Gate::open(&args))
        .and_then(Gate::check);
    let (code, verdict) = match outcome {
        Ok(verdict) => match format {
            Format::Text => (0, verdict.text()),
            Format::Json => (0, verdict.json()),
        },
        Err(fail) => match format {
            Format::Text => (fail.exit_code, fail.text()),
            Format::Json => (fail.exit_code, fail.json()),
        },
    };
    println!("{verdict}");
    (code, verdict)
}

/// Калитка для доказательства: код возврата, человекочитаемый вердикт и
/// структурный вердикт (решение 22), если калитка прошла. Печатает прозу на
/// stdout — структура уходит только возвратом.
pub fn run_for_proof(args: &[OsString]) -> (u8, String, Option<Verdict>) {
    let outcome = Args::parse(args)
        .and_then(|args| Gate::open(&args))
        .and_then(Gate::check);
    match outcome {
        Ok(verdict) => {
            let prose = verdict.text();
            println!("{prose}");
            (0, prose, Some(verdict))
        }
        Err(fail) => {
            let prose = fail.text();
            println!("{prose}");
            (fail.exit_code, prose, None)
        }
    }
}

/// cargo в каталоге `dir` с каталогом сборки `build`. Отсутствующий тулчейн —
/// отказ, а не загрузка внутри хука.
pub fn cargo_command(dir: &Path, build: &Path) -> Command {
    let mut command = tree_command("cargo", dir);
    command
        .env("CARGO_TARGET_DIR", build)
        .env("RUSTUP_AUTO_INSTALL", "0");
    command
}

/// Программа, запускаемая калиткой в каталоге `dir`, с гигиеной окружения.
///
/// Переменные GIT_* хука снимаются: иначе запись дочернего git ушла бы в индекс
/// коммита, ради которого запущен хук. RUSTUP_TOOLCHAIN тоже: rustup передаёт
/// её всем дочерним процессам `cargo run` хука, и она перекрыла бы
/// `rust-toolchain.toml` дерева — коммит, меняющий тулчейн, проверялся бы
/// старым.
fn tree_command(program: &str, dir: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env_remove("RUSTUP_TOOLCHAIN_SOURCE");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
}

/// Сколько шагов полного яруса закрытия работы исполняет калитка поверх яруса
/// коммита: сборка на минимальной версии, атаки на ней и проверка зависимостей
/// (решение 33).
const FULL_TIER_STEPS: usize = 3;

/// Общее число шагов калитки: команда проекта заменяет шесть стандартных шагов
/// cargo и добавляет себя. Число в вердикте обязано быть правдой. Ярус коммита
/// не исполняет полный ярус, и его шаги вычитаются.
fn total(config: &Config, full: bool) -> usize {
    let base = if config.gate_command.is_some() {
        DACC_TOTAL - DELEGATED_STEPS + 1
    } else {
        DACC_TOTAL
    };
    if full {
        base
    } else if config.gate_command.is_some() {
        // В делегированном случае сборка MSRV и зависимости заменены командой
        // проекта; из полного яруса остаётся только атака на минимальной версии.
        base - 1
    } else {
        base - FULL_TIER_STEPS
    }
}

/// Имена внешних проектов из локального списка: без пустых строк и
/// комментариев, в том виде, в каком записаны. Нет списка — пустой список.
pub fn read_external_names(list: &Path) -> Vec<String> {
    fs::read_to_string(list)
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Установлена ли программа: она запускается и отвечает на `--version`.
pub fn installed(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Содержимое файла как текст; нечитаемый файл — пустая строка.
pub fn read_lossy(path: &Path) -> String {
    fs::read(path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// Отказ калитки: шаг, стабильный код причины и код возврата.
struct Fail {
    step: String,
    refusal: Code,
    exit_code: u8,
}

impl Fail {
    /// Человекочитаемый вердикт отказа: текст шага и код причины рядом.
    fn text(&self) -> String {
        format!("GATE FAIL: {} [{}]", self.step, self.refusal)
    }

    /// Машинный вердикт отказа: код причины, шаг и код возврата полями.
    fn json(&self) -> String {
        format!(
            "{{\"ok\":false,\"code\":{},\"step\":{},\"exit_code\":{}}}",
            format::string(self.refusal),
            format::string(&self.step),
            self.exit_code
        )
    }
}

/// Структурный вердикт калитки (решение 22): поля, а не предложение.
pub struct Verdict {
    /// Число пройденных шагов.
    pub passed: usize,
    /// Общее число шагов.
    pub total: usize,
    /// Невыполненные шаги (без предмета проверки).
    pub skipped: Vec<&'static str>,
    /// Полная калитка: число прошедших атак.
    pub attacks: Option<usize>,
    /// Полная калитка: минимальный тулчейн, на котором проверяли.
    pub msrv: Option<String>,
    /// Полная калитка: число прошедших атак на минимальном тулчейне.
    pub msrv_attacks: Option<usize>,
    /// Калитка журнала: дерево без журнала уже проверено.
    pub journal_only: bool,
}

impl Verdict {
    /// Человекочитаемый вердикт: `GATE OK (<n> of <m>; …)`. Когда шаги не
    /// выполнялись, счёт не обещает полного числа шагов — он явно называет
    /// «K not run», чтобы знаменатель не читался как «все шаги проверены».
    pub fn text(&self) -> String {
        let count = if self.skipped.is_empty() {
            format!("{} of {}", self.passed, self.total)
        } else {
            format!(
                "{} of {}, {} not run",
                self.passed,
                self.total,
                self.skipped.len()
            )
        };
        let mut verdict = if self.journal_only {
            format!(
                "GATE OK ({count}; journal only — the tree without the journal is already checked"
            )
        } else if let (Some(msrv), Some(msrv_attacks)) = (&self.msrv, &self.msrv_attacks) {
            format!(
                "GATE OK ({count}; attacks {}, on {} — {}",
                self.attacks.unwrap_or(0),
                msrv,
                msrv_attacks
            )
        } else {
            // Ярус коммита (решение 33): атаки есть, MSRV не проверялся.
            format!("GATE OK ({count}; attacks {}", self.attacks.unwrap_or(0))
        };
        if !self.skipped.is_empty() {
            verdict.push_str("; not run: ");
            verdict.push_str(&self.skipped.join(", "));
        }
        verdict.push(')');
        verdict
    }

    /// Машинный вердикт: пройденные и общее число шагов, атаки, тулчейн и
    /// невыполненные шаги полями. `msrv` и `msrv_attacks` есть только у полного
    /// яруса (решение 33): ярус коммита их не несёт, и поля отсутствуют.
    pub fn json(&self) -> String {
        let mut verdict = format!(
            "{{\"ok\":true,\"passed\":{},\"total\":{}",
            self.passed, self.total
        );
        if self.journal_only {
            verdict.push_str(",\"journal_only\":true");
        } else {
            verdict.push_str(&format!(",\"attacks\":{}", self.attacks.unwrap_or(0)));
            if let Some(msrv) = &self.msrv {
                verdict.push_str(&format!(",\"msrv\":{}", format::string(msrv)));
            }
            if let Some(msrv_attacks) = self.msrv_attacks {
                verdict.push_str(&format!(",\"msrv_attacks\":{msrv_attacks}"));
            }
        }
        if !self.skipped.is_empty() {
            verdict.push_str(",\"skipped\":[");
            verdict.push_str(
                &self
                    .skipped
                    .iter()
                    .map(|step| format::string(step))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            verdict.push(']');
        }
        verdict.push('}');
        verdict
    }
}

/// Шаг упал.
fn fail(step: impl Into<String>) -> Fail {
    Fail {
        step: step.into(),
        refusal: code::STEP_FAILED,
        exit_code: 1,
    }
}

/// Калитку не из чего или нечем выполнить.
fn start_fail(step: impl Into<String>) -> Fail {
    Fail {
        step: step.into(),
        refusal: code::START_ERROR,
        exit_code: 2,
    }
}

struct Args {
    repo: Option<PathBuf>,
    tree: Option<PathBuf>,
    journal_only: bool,
    full: bool,
}

impl Args {
    fn parse(args: &[OsString]) -> Result<Args, Fail> {
        let mut parsed = Args {
            repo: None,
            tree: None,
            journal_only: false,
            full: false,
        };
        let mut rest = args;
        while let Some((first, tail)) = rest.split_first() {
            match first.to_str() {
                Some("--repo") => {
                    let (dir, tail) = tail
                        .split_first()
                        .ok_or_else(|| start_fail("--repo needs a directory"))?;
                    parsed.repo = Some(PathBuf::from(dir));
                    rest = tail;
                }
                Some("--journal-only") => {
                    parsed.journal_only = true;
                    rest = tail;
                }
                Some("--full") => {
                    parsed.full = true;
                    rest = tail;
                }
                Some("--format") => {
                    let (value, tail) = tail
                        .split_first()
                        .ok_or_else(|| start_fail("--format needs `json` or `text`"))?;
                    let value = value
                        .to_str()
                        .ok_or_else(|| start_fail("--format value is not UTF-8"))?;
                    format::parse(value).map_err(start_fail)?;
                    rest = tail;
                }
                _ if parsed.tree.is_none() => {
                    parsed.tree = Some(PathBuf::from(first));
                    rest = tail;
                }
                _ => {
                    return Err(start_fail(format!(
                        "extra argument {}",
                        first.to_string_lossy()
                    )))
                }
            }
        }
        Ok(parsed)
    }
}

struct Gate {
    root: PathBuf,
    tree: PathBuf,
    common_dir: PathBuf,
    target: PathBuf,
    config: Config,
    journal_only: bool,
    full: bool,
    passed: usize,
    skipped: Vec<&'static str>,
}

impl Gate {
    fn open(args: &Args) -> Result<Gate, Fail> {
        let start = args.repo.as_deref().unwrap_or(Path::new("."));
        let repo = git::Repo::discover(start).map_err(start_fail)?;
        let tree = args.tree.clone().unwrap_or_else(|| repo.root.clone());
        let tree = fs::canonicalize(&tree)
            .map_err(|_| start_fail(format!("no directory {}", tree.display())))?;
        // Настройка — из проверяемого дерева, как Cargo.toml и deny.toml: иначе
        // правка настройки в рабочей копии меняла бы вердикт для чужого дерева
        // (решение 20).
        let config = Config::read_dir(&tree).map_err(start_fail)?;
        let target = repo.root.join(layout::GATE_TARGET);
        fs::create_dir_all(&target)
            .map_err(|_| start_fail(format!("cannot create {}", target.display())))?;
        Ok(Gate {
            root: repo.root,
            tree,
            common_dir: repo.common_dir,
            target,
            config,
            journal_only: args.journal_only,
            full: args.full,
            passed: 0,
            skipped: Vec::new(),
        })
    }

    fn check(mut self) -> Result<Verdict, Fail> {
        self.external_names()?;
        self.markdown_links()?;
        self.journal()?;
        if self.journal_only {
            return self.check_journal_build();
        }
        // Команда проекта — сразу после журнала и до шагов cargo: дешёвые
        // проверки DACC отказывают первыми, а команда проекта обычно сама
        // включает fmt, clippy и тесты, поэтому эти шаги здесь пропускаются
        // (решение 28). Признак делегирования вычисляется один раз, чтобы шесть
        // стандартных шагов cargo не повторяли одно и то же условие.
        let delegated = self.config.gate_command.is_some();
        self.project_command()?;

        // Дальше запускается cargo. Без манифеста в самом дереве cargo пошёл бы
        // искать рабочее пространство в родительских каталогах и собрал бы
        // чужой проект, поэтому манифест обязателен и передаётся явно.
        let manifest = self.tree.join(layout::MANIFEST);
        if !manifest.is_file() {
            return Err(start_fail(
                "no Cargo.toml in the tree — the build does not start",
            ));
        }
        let target = self.target.clone();

        if !delegated {
            let mut fmt = self.cargo(&target);
            fmt.args(["fmt", "--manifest-path"])
                .arg(&manifest)
                .args(["--all", "--check"]);
            self.cargo_step("cargo fmt --check", "fmt", &mut fmt)?;
        }

        if !delegated {
            let mut clippy = self.cargo(&target);
            clippy
                .args(["clippy", "--manifest-path"])
                .arg(&manifest)
                .args([
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ]);
            self.cargo_step("cargo clippy -D warnings", "clippy", &mut clippy)?;
        }

        // Doctest-атаки выполняются отдельным шагом под RUSTC_BOOTSTRAP.
        if !delegated {
            let mut test = self.cargo(&target);
            test.args(["test", "--manifest-path"]).arg(&manifest).args([
                "--workspace",
                "--all-targets",
                "--no-fail-fast",
                "--locked",
            ]);
            self.cargo_step("cargo test --all-targets", "test", &mut test)?;
        }

        // rustdoc сверяет коды compile_fail только в nightly-режиме; на stable его
        // включает RUSTC_BOOTSTRAP=1 (решение 12). Без сверки атака прошла бы на
        // любой ошибке компиляции, поэтому механизм проверяется до атак.
        self.write_probe()?;
        self.probe_codes("probe", &target.join("probe-build"), None)?;
        self.passed += 1;

        // Отдельный каталог сборки: build.rs зависимостей следят за
        // RUSTC_BOOTSTRAP, и общий кэш пересобирался бы на каждом шаге.
        let mut attacks = self.cargo(&self.build_dir("attacks"));
        attacks
            .env("RUSTC_BOOTSTRAP", "1")
            .args(["test", "--manifest-path"])
            .arg(&manifest)
            .args(["--workspace", "--doc", "--no-fail-fast", "--locked"]);
        let log = self.cargo_step("attacks: cargo test --doc", "attacks", &mut attacks)?;
        // Пол — из настройки проверяемого дерева: число атак у каждого проекта
        // своё, и зашитое число DACC отказывало бы в чужом всегда
        // (решение 20).
        let floor = self.config.doctest_floor;
        let attacks = count_attacks(&log);
        if attacks < floor {
            return Err(fail(format!(
                "attacks passed {attacks} at floor {floor} — attacks were not run or were removed"
            )));
        }

        if !delegated {
            let mut doc = self.cargo(&target);
            doc.env("RUSTDOCFLAGS", "-D warnings")
                .args(["doc", "--manifest-path"])
                .arg(&manifest)
                .args(["--workspace", "--no-deps", "--locked"]);
            self.cargo_step("cargo doc -D warnings", "doc", &mut doc)?;
        }

        // Полный ярус (решение 33): MSRV и зависимости — только при приземлении
        // работы. Ярус коммита их пропускает, и вердикт остаётся без msrv.
        let (msrv, msrv_attacks) = if self.full {
            // Обещанная потребителям невыразимость проверяется на обещанном им
            // компиляторе (решение 12).
            let msrv = fs::read_to_string(&manifest)
                .ok()
                .and_then(|text| minimum_rust(&text))
                .ok_or_else(|| {
                    start_fail(
                        "no rust-version in the tree's Cargo.toml — the minimum version is not checked",
                    )
                })?;
            let toolchain = format!("+{msrv}");

            if !delegated {
                let mut msrv_check = self.cargo(&self.build_dir("msrv"));
                msrv_check
                    .arg(&toolchain)
                    .args(["check", "--manifest-path"])
                    .arg(&manifest)
                    .args(["--workspace", "--all-targets", "--locked"]);
                let label = format!("cargo {toolchain} check --all-targets");
                self.cargo_step(&label, "msrv-check", &mut msrv_check)?;
            }

            self.probe_codes(
                "msrv-probe",
                &self.build_dir("msrv-probe"),
                Some(&toolchain),
            )?;
            let mut msrv_attacks = self.cargo(&self.build_dir("msrv-attacks"));
            msrv_attacks
                .arg(&toolchain)
                .env("RUSTC_BOOTSTRAP", "1")
                .args(["test", "--manifest-path"])
                .arg(&manifest)
                .args(["--workspace", "--doc", "--no-fail-fast", "--locked"]);
            let label = format!("attacks on {msrv}: cargo test --doc");
            let log = self.cargo_step(&label, "msrv-attacks", &mut msrv_attacks)?;
            let msrv_attacks = count_attacks(&log);
            if msrv_attacks < floor {
                return Err(fail(format!(
                    "on {msrv} attacks passed {msrv_attacks} at floor {floor} — attacks were not run or were removed"
                )));
            }

            // Политика по сохранённой базе, без сети: коммит от сети не зависит.
            // Базу обновляет pre-push; без базы шаг отказывает (решение 13). Шаг
            // заменяется командой проекта, когда она задана (решение 28).
            if !delegated {
                let policy = self.tree.join(layout::DENY_POLICY);
                if !policy.is_file() {
                    return Err(fail(
                        "no deny.toml in the tree — no dependency policy is set (decision 13)",
                    ));
                }
                if !installed("cargo-deny") {
                    return Err(start_fail(
                        "cargo-deny is not installed — cargo install cargo-deny --locked",
                    ));
                }
                let mut deny = self.cargo(&target);
                deny.args(["deny", "--manifest-path"])
                    .arg(&manifest)
                    .arg("--config")
                    .arg(&policy)
                    .args(["--frozen", "check"]);
                self.cargo_step("cargo deny check", "deny", &mut deny)?;
            }
            (Some(msrv), Some(msrv_attacks))
        } else {
            (None, None)
        };

        Ok(Verdict {
            passed: self.passed,
            total: total(&self.config, self.full),
            skipped: self.skipped,
            attacks: Some(attacks),
            msrv,
            msrv_attacks,
            journal_only: false,
        })
    }

    /// Калитка журнала: дерево без журнала уже проверено, осталось собрать
    /// крейт документов — сборка сворачивает журнал (решение 15).
    fn check_journal_build(mut self) -> Result<Verdict, Fail> {
        let manifest = self.tree.join(self.config.doc_manifest());
        if !manifest.is_file() {
            return Err(start_fail(format!(
                "no {} in the tree — the journal cannot be folded",
                self.config.doc_manifest()
            )));
        }
        let target = self.target.clone();
        let mut build = self.cargo(&target);
        build
            .args(["check", "--manifest-path"])
            .arg(&manifest)
            .arg("--locked");
        self.cargo_step("journal fold: cargo check doc", "journal-check", &mut build)?;
        Ok(Verdict {
            passed: self.passed,
            total: JOURNAL_ONLY_TOTAL,
            skipped: self.skipped,
            attacks: None,
            msrv: None,
            msrv_attacks: None,
            journal_only: true,
        })
    }

    /// Делегированный шаг: команда проекта из настройки проверяемого дерева.
    /// Её отказ — отказ калитки, и текст называет команду и код возврата. Когда
    /// команды нет, стандартные шаги cargo выполняет сама калитка; когда есть —
    /// команда заменяет их (решение 28).
    fn project_command(&mut self) -> Result<(), Fail> {
        let Some(words) = self.config.gate_command.clone() else {
            return Ok(());
        };
        let (program, args) = words
            .split_first()
            .ok_or_else(|| start_fail("gate_command is empty"))?;
        // Команда выполняется в проверяемом дереве и с той же гигиеной
        // окружения, что и cargo: её дочерний git не пишет в индекс коммита,
        // ради которого запущен хук.
        let mut command = tree_command(program, &self.tree);
        command.args(args);
        let label = format!("project command: {}", words.join(" "));
        self.cargo_step(&label, "project", &mut command)?;
        Ok(())
    }

    /// Шаг 1: имена внешних проектов не встречаются в файлах дерева.
    fn external_names(&mut self) -> Result<(), Fail> {
        let list = self.common_dir.join("info").join(layout::EXTERNAL_NAMES);
        let patterns: Vec<String> = read_external_names(&list)
            .iter()
            .map(|name| name.to_lowercase())
            .collect();
        if patterns.is_empty() {
            println!(
                "external names: list {} is empty or not set — step not run",
                list.display()
            );
            self.skipped.push("external names");
            return Ok(());
        }
        let broken =
            |error: std::io::Error| start_fail(format!("external name search failed: {error}"));
        let mut hits = Vec::new();
        for file in walk(&self.tree, &self.root).map_err(broken)? {
            let bytes = fs::read(&file).map_err(broken)?;
            // Двоичный файл пропускается, как у grep -I.
            if bytes.iter().take(8000).any(|&b| b == 0) {
                continue;
            }
            let text = String::from_utf8_lossy(&bytes).to_lowercase();
            if patterns.iter().any(|name| text.contains(name.as_str())) {
                hits.push(relative(&self.tree, &file));
            }
        }
        if !hits.is_empty() {
            for hit in &hits {
                println!("  external name in file: {hit}");
            }
            return Err(fail("external names in the tree (decision 9)"));
        }
        self.passed += 1;
        Ok(())
    }

    /// Шаг 2: относительные ссылки в markdown ведут в существующие файлы.
    fn markdown_links(&mut self) -> Result<(), Fail> {
        let broken_walk = |error: std::io::Error| start_fail(format!("tree walk failed: {error}"));
        let documents: Vec<PathBuf> = walk(&self.tree, &self.root)
            .map_err(broken_walk)?
            .into_iter()
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();
        if documents.is_empty() {
            self.skipped.push("markdown links");
            return Ok(());
        }
        let mut broken = Vec::new();
        for document in &documents {
            let text = read_lossy(document);
            let dir = document.parent().unwrap_or(&self.tree);
            for link in relative_links(&text) {
                let path = link.split('#').next().unwrap_or_default();
                if !path.is_empty() && !dir.join(path).exists() {
                    broken.push(format!("{}: {link}", relative(&self.tree, document)));
                }
            }
        }
        if !broken.is_empty() {
            for link in &broken {
                println!("  broken link: {link}");
            }
            return Err(fail("relative links in markdown"));
        }
        self.passed += 1;
        Ok(())
    }

    /// Шаг 3: файлы событий из HEAD не изменены и не удалены, а каждое
    /// приземление ссылается на существующий коммит, дерево которого без
    /// журнала совпадает с деревом события. Законность переходов проверяет
    /// сборка крейта документов.
    fn journal(&mut self) -> Result<(), Fail> {
        let dir = self.tree.join(self.config.journal_dir());
        let file = self.tree.join(self.config.journal_file());
        if !dir.is_dir() && !file.is_file() {
            self.skipped.push("journal");
            return Ok(());
        }
        let mut problems = self.changed_event_files();
        problems.extend(self.changed_journal_file());
        // Одна запись и прежний каталог читаются вместе (решение 43).
        let (file_events, _) = dacc_journal::read_file(&file)
            .map_err(|error| start_fail(format!("journal not read: {error}")))?;
        let (dir_events, _) = if dir.is_dir() {
            dacc_journal::read_dir(&dir)
                .map_err(|error| start_fail(format!("journal not read: {error}")))?
        } else {
            (Vec::new(), Vec::new())
        };
        let mut events = file_events;
        events.extend(dir_events);
        for event in &events {
            let Kind::Landed { commit, tree, .. } = &event.kind else {
                continue;
            };
            let object = format!("{commit}^{{commit}}");
            if !git::succeeds(&self.root, &["cat-file", "-e", &object]) {
                problems.push(format!(
                    "{}: commit {commit} is not in the repository",
                    event.file
                ));
            } else if proof::content_hash(&self.root, &self.config.journal_dir(), commit).as_deref()
                != Some(tree.as_str())
            {
                problems.push(format!(
                    "{}: the tree of commit {commit} without the journal differs from the event tree",
                    event.file
                ));
            }
        }
        if !problems.is_empty() {
            for problem in &problems {
                println!("  journal: {problem}");
            }
            return Err(fail("journal differs from history (decision 15)"));
        }
        self.passed += 1;
        Ok(())
    }

    /// Файлы событий из HEAD, изменённые или удалённые в дереве. Без HEAD —
    /// первый коммит — сравнивать не с чем.
    fn changed_event_files(&self) -> Vec<String> {
        let journal = self.config.journal_dir();
        let Some(listing) = git::read(&self.root, &["ls-tree", "-r", "-z", "HEAD", "--", &journal])
        else {
            return Vec::new();
        };
        let mut problems = Vec::new();
        let mut present = Vec::new();
        for entry in listing.split('\0') {
            let Some((meta, path)) = entry.split_once('\t') else {
                continue;
            };
            let Some(sha) = meta.split_whitespace().nth(2) else {
                continue;
            };
            if !path.ends_with(".toml") {
                continue;
            }
            let file = self.tree.join(path);
            if file.is_file() {
                present.push((sha.to_owned(), path.to_owned(), file));
            } else {
                problems.push(format!("{path}: event file deleted"));
            }
        }
        let files: Vec<PathBuf> = present.iter().map(|(_, _, file)| file.clone()).collect();
        let hashes = proof::file_hashes(&self.root, &files);
        for ((sha, path, _), hash) in present.iter().zip(hashes) {
            if hash.as_deref() != Some(sha.as_str()) {
                problems.push(format!("{path}: event file changed"));
            }
        }
        problems
    }

    /// Одна запись журнала из HEAD (решение 43): прежняя версия — префикс новой,
    /// поэтому событие не переписывается, а только дописывается. Нет файла в
    /// HEAD — первый коммит — сравнивать не с чем.
    fn changed_journal_file(&self) -> Vec<String> {
        let journal = self.config.journal_file();
        let Some(head) = git::read(&self.root, &["show", &format!("HEAD:{journal}")]) else {
            return Vec::new();
        };
        let text = fs::read_to_string(self.tree.join(&journal)).unwrap_or_default();
        if !text.starts_with(&head) {
            return vec![format!(
                "{journal}: the journal file is not append-only (the committed version is not a prefix)"
            )];
        }
        Vec::new()
    }

    /// cargo из дерева с каталогом сборки `build`.
    fn cargo(&self, build: &Path) -> Command {
        cargo_command(&self.tree, build)
    }

    /// Каталог сборки рядом с основным: `target/gate-<суффикс>`.
    fn build_dir(&self, suffix: &str) -> PathBuf {
        let mut name = self.target.clone().into_os_string();
        name.push("-");
        name.push(suffix);
        PathBuf::from(name)
    }

    /// Запускает шаг с выводом в журнал; при отказе показывает строки ошибок и
    /// хвост журнала. Возвращает путь журнала.
    fn cargo_step(
        &mut self,
        label: &str,
        log: &str,
        command: &mut Command,
    ) -> Result<PathBuf, Fail> {
        let log = self.target.join(format!("{log}.log"));
        let status = run_logged(command, &log)?;
        if !status.success() {
            show_failure(&log);
            let code = status
                .code()
                .map_or_else(|| "signal".to_owned(), |code| code.to_string());
            return Err(fail(format!("{label} (code {code})")));
        }
        self.passed += 1;
        Ok(log)
    }

    /// Файлы пробы; перезаписываются только при изменении, иначе проба
    /// пересобиралась бы на каждом прогоне.
    fn write_probe(&self) -> Result<(), Fail> {
        let probe = self.target.join("probe");
        let write = |path: PathBuf, text: &str| -> Result<(), Fail> {
            if fs::read(&path).is_ok_and(|current| current == text.as_bytes()) {
                return Ok(());
            }
            fs::create_dir_all(path.parent().unwrap_or(&probe))
                .and_then(|()| fs::write(&path, text))
                .map_err(|error| start_fail(format!("probe not written: {error}")))
        };
        write(probe.join("Cargo.toml"), PROBE_MANIFEST)?;
        write(probe.join("src").join("lib.rs"), PROBE_LIB)
    }

    /// Проба сверки кодов тулчейном дерева или явным `toolchain`.
    fn probe_codes(&self, log: &str, build: &Path, toolchain: Option<&str>) -> Result<(), Fail> {
        let mut command = self.cargo(build);
        if let Some(toolchain) = toolchain {
            command.arg(toolchain);
        }
        command
            .env("RUSTC_BOOTSTRAP", "1")
            .args(["test", "--manifest-path"])
            .arg(self.target.join("probe").join("Cargo.toml"))
            .arg("--doc");
        let log = self.target.join(format!("{log}.log"));
        run_logged(&mut command, &log)?;
        let text = read_lossy(&log);
        let wrong_code_refused = text.contains("Some expected error codes were not found");
        let one_of_each = text
            .lines()
            .any(|line| line.starts_with("test result: FAILED. 1 passed; 1 failed;"));
        if wrong_code_refused && one_of_each {
            return Ok(());
        }
        show_failure(&log);
        let on = toolchain.map(|t| format!(" on {t}")).unwrap_or_default();
        Err(fail(format!(
            "error code matching in attacks does not work{on} — attacks would pass vacuously (decision 12)"
        )))
    }
}

/// Запускает команду с выводом в журнал `log`.
fn run_logged(command: &mut Command, log: &Path) -> Result<ExitStatus, Fail> {
    let not_opened =
        |error: std::io::Error| start_fail(format!("log {} not opened: {error}", log.display()));
    let out = File::create(log).map_err(not_opened)?;
    let err = out.try_clone().map_err(not_opened)?;
    command
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .status()
        .map_err(|error| {
            start_fail(format!(
                "{} did not start: {error}",
                command.get_program().to_string_lossy()
            ))
        })
}

/// Строки ошибок, затем хвост журнала и путь к нему.
fn show_failure(log: &Path) {
    let text = read_lossy(log);
    let lines: Vec<&str> = text.lines().collect();
    for line in lines
        .iter()
        .filter(|line| is_error_line(line))
        .take(ERROR_LINES)
    {
        println!("{line}");
    }
    for line in &lines[lines.len().saturating_sub(TAIL_LINES)..] {
        println!("{line}");
    }
    println!("full output: {}", log.display());
}

/// Путь файла от корня дерева — для сообщений.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Файлы дерева по порядку. Для рабочего дерева и подкаталога репозитория —
/// список git (`ls-files`): отслеживаемые и неигнорируемые неотслеживаемые
/// файлы, поэтому `.gitignore` уважается. Выгруженное дерево коммита лежит
/// внутри каталога git и обходится как прежде — там игнорируемых файлов нет по
/// построению, а обычный обход пропускает только `target` и `.git` по имени на
/// любом уровне, не по подстроке пути.
fn walk(root: &Path, repo_root: &Path) -> std::io::Result<Vec<PathBuf>> {
    if root == repo_root {
        return Ok(git_files(
            repo_root,
            &[
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ],
        ));
    }
    if let Ok(rel) = root.strip_prefix(repo_root) {
        let rel = rel.to_string_lossy();
        // Подкаталог репозитория (например, `gate --repo <root> <dir>`): тот же
        // список git, ограниченный этим каталогом, поэтому `.gitignore` в нём
        // уважается. Каталог git не подкаталог проверки: он начинается с `.git/`.
        if !rel.is_empty() && rel != ".git" && !rel.starts_with(".git/") {
            return Ok(git_files(
                repo_root,
                &[
                    "ls-files",
                    "--cached",
                    "--others",
                    "--exclude-standard",
                    "-z",
                    "--",
                    &rel,
                ],
            ));
        }
    }

    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                let name = entry.file_name();
                if !matches!(name.to_str(), Some("target" | ".git")) {
                    dirs.push(entry.path());
                }
            } else if kind.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Файлы от `git ls-files` в `repo_root`: пути вывода относительны ему, поэтому
/// каждый путь пристыковывается к `repo_root`. Без вывода git — пустой список.
fn git_files(repo_root: &Path, args: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(listing) = git::read(repo_root, args) {
        for path in listing.split('\0').filter(|p| !p.is_empty()) {
            files.push(repo_root.join(path));
        }
    }
    files.sort();
    files
}

/// Цели ссылок `](цель)` без пробелов, кроме внешних адресов и якорей.
///
/// Код документа ссылкой не считается: в тексте о правилах коммитов форма
/// темы — пример, а не путь к файлу (работа 31).
fn relative_links(text: &str) -> Vec<&str> {
    let mut links = Vec::new();
    for prose in prose_segments(text) {
        let mut rest = prose;
        while let Some(at) = rest.find("](") {
            rest = &rest[at + 2..];
            let end = rest
                .find(|c: char| c == ')' || c.is_whitespace())
                .unwrap_or(rest.len());
            let target = &rest[..end];
            let external = ["http:", "https:", "mailto:", "#"]
                .iter()
                .any(|prefix| target.starts_with(prefix));
            if rest[end..].starts_with(')') && !target.is_empty() && !external {
                links.push(target);
            }
        }
    }
    links
}

/// Куски текста вне кода: огороженные блоки и код в обратных кавычках
/// выброшены.
///
/// Ограда — три и больше знаков `` ` `` или `~` в начале строки; закрывающая
/// ограда того же знака и не короче открывающей, а незакрытая съедает текст до
/// конца, как в CommonMark. Отступ в четыре пробела кодом здесь не считается:
/// в документах DACC код пишут оградой.
fn prose_segments(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let mark = trimmed.chars().next().filter(|c| *c == '`' || *c == '~');
        if let Some(mark) = mark {
            let run = trimmed.chars().take_while(|c| *c == mark).count();
            if run >= 3 {
                match fence {
                    Some((open, len)) if open == mark && run >= len => {
                        fence = None;
                        continue;
                    }
                    Some(_) => {}
                    None => {
                        fence = Some((mark, run));
                        continue;
                    }
                }
            }
        }
        if fence.is_none() {
            segments.extend(outside_code_spans(line));
        }
    }
    segments
}

/// Куски строки вне кода в обратных кавычках: код закрывается таким же числом
/// кавычек подряд, а незакрытый — концом строки.
fn outside_code_spans(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = line.as_bytes();
    let mut at = 0;
    let mut start = 0;
    while at < bytes.len() {
        if bytes[at] != b'`' {
            at += 1;
            continue;
        }
        let open = bytes[at..].iter().take_while(|byte| **byte == b'`').count();
        parts.push(&line[start..at]);
        let mut cursor = at + open;
        let close = loop {
            let Some(next) = line[cursor..].find('`') else {
                break None;
            };
            let found = cursor + next;
            let run = bytes[found..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if run == open {
                break Some(found + run);
            }
            cursor = found + run;
        };
        match close {
            Some(end) => {
                at = end;
                start = end;
            }
            None => {
                at = bytes.len();
                start = bytes.len();
            }
        }
    }
    parts.push(&line[start..]);
    parts
}

/// Минимальная версия из `rust-version = "1.83"` в виде тулчейна `1.83.0`.
fn minimum_rust(manifest: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let version = line.strip_prefix("rust-version = \"")?.strip_suffix('"')?;
        let parts: Vec<&str> = version.split('.').collect();
        let numeric = parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
        match (numeric, parts.len()) {
            (true, 2) => Some(format!("{version}.0")),
            (true, 3) => Some(version.to_owned()),
            _ => None,
        }
    })
}

/// Число прошедших атакующих doctest в журнале cargo test: только doctest,
/// объявленные падающими при компиляции (работа 46). Обычные примеры пола не
/// набирают — иначе обещание «атаки не удалены» держалось бы доброй волей.
fn count_attacks(log: &Path) -> usize {
    read_lossy(log)
        .lines()
        .filter(|line| is_passed_attack(line))
        .count()
}

/// `test <файл> - <элемент> (line N) - compile fail ... ok`.
fn is_passed_attack(line: &str) -> bool {
    let Some(rest) = line
        .strip_prefix("test ")
        .and_then(|rest| rest.strip_suffix(" ... ok"))
    else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(" - compile fail") else {
        return false;
    };
    let Some(open) = rest.rfind("(line ") else {
        return false;
    };
    let Some(number) = rest[open + "(line ".len()..].strip_suffix(')') else {
        return false;
    };
    let head = &rest[..open];
    !number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit())
        && head
            .split_once(" - ")
            .is_some_and(|(file, item)| !file.is_empty() && !item.trim().is_empty())
}

/// Строка журнала cargo, которую стоит показать при отказе шага.
fn is_error_line(line: &str) -> bool {
    let tagged = ["error", "warning", "bug"].into_iter().any(|kind| {
        let Some(rest) = line.strip_prefix(kind) else {
            return false;
        };
        let rest = match rest.strip_prefix('[') {
            Some(code) => match code.split_once(']') {
                Some((code, rest))
                    if !code.is_empty()
                        && code
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') =>
                {
                    rest
                }
                _ => return false,
            },
            None => rest,
        };
        rest.starts_with(':')
    });
    tagged
        || (line.starts_with("test ") && line.ends_with(" FAILED"))
        || line.starts_with("---- ")
        || line.starts_with("Diff in ")
        || line.starts_with("Some expected error codes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_attacks_are_counted_toward_the_floor() {
        // Обычный doctest чтения реестра пола атак не набирает (работа 46).
        assert!(!is_passed_attack(
            "test crates/dacc-core/src/nonempty.rs - nonempty::NonEmpty (line 12) ... ok"
        ));
        // Атакующий doctest — набирает.
        assert!(is_passed_attack(
            "test crates/dacc-knowledge/src/attacks.rs - attacks (line 11) - compile fail ... ok"
        ));
        for line in [
            "test attacks::e1 ... ok",
            "test crates/a.rs - x (line 3) ... FAILED",
            "test crates/a.rs - x (line three) - compile fail ... ok",
            "test result: ok. 20 passed; 0 failed",
        ] {
            assert!(!is_passed_attack(line), "{line}");
        }
    }

    #[test]
    fn error_lines_are_recognised() {
        for line in [
            "error[E0080]: evaluation of constant value failed",
            "error[wildcard]: found 3 wildcard dependencies",
            "error: could not compile `dacc-cli`",
            "warning: unused import",
            "bug[unresolved-workspace-dependency]: failed to resolve",
            "test attacks::e1 ... FAILED",
            "---- attacks::e1 stdout ----",
            "Diff in /tree/src/lib.rs:3:",
            "Some expected error codes were not found: [\"E0080\"]",
        ] {
            assert!(is_error_line(line), "{line}");
        }
        for line in [
            "   Compiling dacc-core",
            "errors: none",
            "error[]: x",
            "test x ... ok",
        ] {
            assert!(!is_error_line(line), "{line}");
        }
    }

    #[test]
    fn only_relative_links_are_checked() {
        let text =
            "[a](b.md) [c](https://x.org) [d](#якорь) [e](dir/f.md#раздел) [g](with space) [h]()";
        assert_eq!(relative_links(text), ["b.md", "dir/f.md#раздел"]);
    }

    #[test]
    fn code_in_a_document_is_not_a_link() {
        // Форма темы коммита в тексте документа — код, а не ссылка на файл
        // «область»: шаг отвергал документ за форму записи (работа 31).
        let text = concat!(
            "тема `[ТИП](область): суть`\n\n",
            "```text\n[FEAT](cli): суть\n```\n\n",
            "а это ссылка: [файл](real.md)\n"
        );
        assert_eq!(relative_links(text), ["real.md"]);
    }

    /// Число шагов в вердикте — правда: команда проекта заменяет шесть
    /// стандартных шагов cargo и добавляет себя (решение 28); ярус коммита не
    /// исполняет полный ярус (решение 33).
    #[test]
    fn the_project_command_replaces_the_cargo_steps_in_the_total() {
        let mut config = Config::default();
        assert_eq!(total(&config, true), DACC_TOTAL);
        assert_eq!(total(&config, false), DACC_TOTAL - FULL_TIER_STEPS);
        config.gate_command = Some(vec!["make".to_owned(), "check".to_owned()]);
        assert_eq!(total(&config, true), DACC_TOTAL - DELEGATED_STEPS + 1);
        assert_eq!(total(&config, false), DACC_TOTAL - DELEGATED_STEPS);
    }

    #[test]
    fn minimum_rust_becomes_a_toolchain() {
        assert_eq!(
            minimum_rust("[workspace.package]\nrust-version = \"1.83\"\n").as_deref(),
            Some("1.83.0")
        );
        assert_eq!(
            minimum_rust("rust-version = \"1.83.2\"").as_deref(),
            Some("1.83.2")
        );
        assert_eq!(minimum_rust("rust-version.workspace = true"), None);
        assert_eq!(minimum_rust("rust-version = \"1.x\""), None);
    }

    /// Снимок обеих форм вердикта полной калитки (решение 22): проза для
    /// человека, поля для машины.
    #[test]
    fn verdict_renders_prose_and_json_fields() {
        let verdict = Verdict {
            passed: 11,
            total: DACC_TOTAL,
            skipped: vec!["external names", "markdown links"],
            attacks: Some(20),
            msrv: Some("1.83.0".to_owned()),
            msrv_attacks: Some(20),
            journal_only: false,
        };
        assert_eq!(
            verdict.text(),
            "GATE OK (11 of 12, 2 not run; attacks 20, on 1.83.0 — 20; not run: external names, markdown links)"
        );
        assert_eq!(
            verdict.json(),
            "{\"ok\":true,\"passed\":11,\"total\":12,\"attacks\":20,\"msrv\":\"1.83.0\",\"msrv_attacks\":20,\"skipped\":[\"external names\",\"markdown links\"]}"
        );
    }

    /// Ярус коммита (решение 33): атаки есть, MSRV не проверялся — проза не
    /// называет тулчейн, JSON не несёт `msrv` и `msrv_attacks`.
    #[test]
    fn commit_tier_verdict_omits_msrv() {
        let verdict = Verdict {
            passed: 8,
            total: DACC_TOTAL - FULL_TIER_STEPS,
            skipped: vec!["external names"],
            attacks: Some(20),
            msrv: None,
            msrv_attacks: None,
            journal_only: false,
        };
        assert_eq!(
            verdict.text(),
            "GATE OK (8 of 9, 1 not run; attacks 20; not run: external names)"
        );
        assert_eq!(
            verdict.json(),
            "{\"ok\":true,\"passed\":8,\"total\":9,\"attacks\":20,\"skipped\":[\"external names\"]}"
        );
    }

    /// Калитка журнала не считает атаки и не называет тулчейн: JSON несёт
    /// признак `journal_only`, а не пустые поля.
    #[test]
    fn journal_only_verdict_omits_attacks_and_msrv() {
        let verdict = Verdict {
            passed: 3,
            total: JOURNAL_ONLY_TOTAL,
            skipped: vec![],
            attacks: None,
            msrv: None,
            msrv_attacks: None,
            journal_only: true,
        };
        assert_eq!(
            verdict.text(),
            "GATE OK (3 of 4; journal only — the tree without the journal is already checked)"
        );
        assert_eq!(
            verdict.json(),
            "{\"ok\":true,\"passed\":3,\"total\":4,\"journal_only\":true}"
        );
    }

    /// Отказ калитки: шаг и код возврата полями, рядом — стабильный код причины.
    #[test]
    fn fail_renders_step_and_code() {
        let fail = Fail {
            step: "cargo clippy -D warnings".to_owned(),
            refusal: code::STEP_FAILED,
            exit_code: 1,
        };
        assert_eq!(
            fail.text(),
            "GATE FAIL: cargo clippy -D warnings [step-failed]"
        );
        assert_eq!(
            fail.json(),
            "{\"ok\":false,\"code\":\"step-failed\",\"step\":\"cargo clippy -D warnings\",\"exit_code\":1}"
        );
    }
}
