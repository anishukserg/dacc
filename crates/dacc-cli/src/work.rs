//! Команды работы и среза (решение 15): события журнала пишет инструмент, а не
//! человек руками, и сразу коммитит их по правилам коммитов.
//!
//! ```text
//! cargo dacc work start <w-slug> [--trailer <trailer>]…
//! cargo dacc work land <w-slug> [--commit <revision>] [--trailer <trailer>]…
//! cargo dacc work drop <w-slug> --reason <reason> [--trailer <trailer>]…
//! cargo dacc work state [<w-slug>] [--format json|text]
//! cargo dacc slice close <s-slug> [--trailer <trailer>]…
//! ```
//!
//! Незаконный переход, отсутствующая работа и приземление без доказательства
//! отвергаются до записи события. Последняя строка — вердикт команды commit или
//! `WORK REFUSED: <reason>`.
//!
//! Пути реестра и темы служебных коммитов — из настройки рабочего дерева
//! (решение 20): команда пишет событие в это дерево и по его правилам.
//!
//! Код возврата: 0 — событие записано и закоммичено или свёртка напечатана;
//! 1 — переход незаконен, нет доказательства или журнал не сворачивается;
//! 2 — неверные аргументы или окружение; прочие коды — коды команды commit.

use crate::code::{self, Code};
use crate::format::{self, Format};
use crate::{commit, config, git, layout, proof};
use dacc_journal::event::relative_path;
use dacc_journal::{fold, time, Event, Evidence, GateVerdict, Journal, Kind, Stage, Subject};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const WORK_USAGE: &str = "work start <w-slug> | new <w-slug> --slice … --origin … | land <w-slug> [--commit <revision>] | drop <w-slug> --reason <reason> | state [<w-slug>]";

const SLICE_USAGE: &str = "slice close <s-slug>";

/// `cargo dacc work …`.
pub fn run_work(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, trailers)| {
        // `--format` относится только к `state`: извлекается из любого места.
        let mut format = Format::Text;
        let mut rest = Vec::new();
        let mut iter = words.into_iter();
        while let Some(word) = iter.next() {
            if word == "--format" {
                let value = iter
                    .next()
                    .ok_or_else(|| usage(code::FORMAT_CHOICE, "--format needs `json` or `text`"))?;
                format =
                    format::parse(&value).map_err(|problem| usage(code::FORMAT_CHOICE, problem))?;
            } else {
                rest.push(word);
            }
        }
        match rest
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["start", id] => start(id, &trailers),
            ["land", id] => land(id, "HEAD", &trailers),
            ["land", id, "--commit", revision] => land(id, revision, &trailers),
            ["drop", id, "--reason", reason] => abandon(id, reason, &trailers),
            ["state"] => state(None, format),
            ["state", id] => state(Some(id), format),
            ["new", tail @ ..] => {
                let args: Vec<String> = tail.iter().map(|word| (*word).to_owned()).collect();
                crate::work_new::work_new(&args)
            }
            _ => Err(usage(code::USAGE, WORK_USAGE)),
        }
    }))
}

/// `cargo dacc slice …`.
pub fn run_slice(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, trailers)| {
        match words
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["close", id] => close(id, &trailers),
            _ => Err(usage(code::USAGE, SLICE_USAGE)),
        }
    }))
}

/// Отказ команды: код возврата, стабильный код причины и текст.
#[derive(Debug)]
pub struct Refusal {
    exit_code: u8,
    refusal: Code,
    reason: String,
}

pub fn refused(refusal: Code, reason: impl Into<String>) -> Refusal {
    Refusal {
        exit_code: 1,
        refusal,
        reason: reason.into(),
    }
}

pub fn usage(refusal: Code, reason: impl Into<String>) -> Refusal {
    Refusal {
        exit_code: 2,
        refusal,
        reason: reason.into(),
    }
}

fn finish(outcome: Result<u8, Refusal>) -> u8 {
    match outcome {
        Ok(code) => code,
        Err(refusal) => {
            println!("WORK REFUSED: {} [{}]", refusal.reason, refusal.refusal);
            refusal.exit_code
        }
    }
}

/// Аргументы словами и строки `--trailer`, извлечённые из любого места.
fn words(args: &[OsString]) -> Result<(Vec<String>, Vec<String>), Refusal> {
    let mut words = Vec::new();
    let mut trailers = Vec::new();
    let mut rest = args.iter().map(|arg| arg.to_string_lossy().into_owned());
    while let Some(word) = rest.next() {
        if word == "--trailer" {
            let trailer = rest.next().ok_or_else(|| {
                usage(code::TRAILER_NEEDS_VALUE, "--trailer needs a trailer line")
            })?;
            if !trailer.contains(": ") {
                return Err(usage(
                    code::TRAILER_KEY_VALUE,
                    format!("trailer `{trailer}` must be of the form `Key: value`"),
                ));
            }
            trailers.push(trailer);
        } else {
            words.push(word);
        }
    }
    Ok((words, trailers))
}

fn start(id: &str, trailers: &[String]) -> Result<u8, Refusal> {
    let context = Context::open()?;
    let number = work_number(id)?;
    let work = context.work(id)?;
    let stage = context.journal.stage(&number);
    if stage != Stage::Planned {
        return Err(refused(
            code::WORK_STAGE,
            format!("work {id} is already {}", stage_text(stage)),
        ));
    }
    if let Some(slice) = &work.slice {
        if let Some(file) = context.journal.closed_slices.get(slice) {
            return Err(refused(
                code::SLICE_ALREADY_CLOSED,
                format!("slice {slice} of work {id} is closed by event {file}"),
            ));
        }
    }
    let area = work.area(id)?;
    let event = Event::new(Subject::Work(number), time::now(), Kind::Started);
    let message = message(
        &config::fill(&context.repo.config.subject_started, area, id),
        &work.title,
        &format!("Dacc-Work: {id}"),
        trailers,
    );
    record_and_commit(&context.repo, vec![event], &message)
}

fn land(id: &str, revision: &str, trailers: &[String]) -> Result<u8, Refusal> {
    let context = Context::open()?;
    let number = work_number(id)?;
    let work = context.work(id)?;
    let stage = context.journal.stage(&number);
    if stage != Stage::Started {
        return Err(refused(
            code::WORK_STAGE,
            format!(
                "only a started work can be landed, and {id} is {}",
                stage_text(stage)
            ),
        ));
    }
    let root = &context.repo.root;
    let object = format!("{revision}^{{commit}}");
    let commit =
        git::read(root, &["rev-parse", "--verify", "--quiet", &object]).ok_or_else(|| {
            refused(
                code::REVISION_NOT_COMMIT,
                format!("revision {revision} is not a commit"),
            )
        })?;
    if !git::succeeds(root, &["merge-base", "--is-ancestor", &commit, "HEAD"]) {
        return Err(refused(
            code::COMMIT_NOT_ANCESTOR,
            format!("commit {} is not in the history of HEAD", short(&commit)),
        ));
    }
    let trailer = format!("Dacc-Work: {id}");
    let body = git::read(root, &["log", "-1", "--format=%B", &commit]).unwrap_or_default();
    if !body.lines().any(|line| line == trailer) {
        return Err(refused(
            code::COMMIT_NOT_BASED,
            format!(
                "commit {} is not based on work {id}: it has no `{trailer}` trailer",
                short(&commit)
            ),
        ));
    }
    let journal = context.repo.config.journal_dir();
    let tree = proof::content_hash(root, &journal, &commit).ok_or_else(|| {
        usage(
            code::TREE_NOT_READ,
            format!("the tree of commit {} cannot be read", short(&commit)),
        )
    })?;
    let verdict = gate_verdict(&context.repo.common_dir, &tree).ok_or_else(|| {
        refused(
            code::NO_PROOF,
            format!(
                "no proof for tree {} of commit {}: the gate did not pass on this tree here — the commit was made without the hook or on another machine",
                short(&tree),
                short(&commit)
            ),
        )
    })?;
    let area = work.area(id)?;
    let at = time::now();
    let gate = Event::new(
        Subject::Work(number.clone()),
        at.clone(),
        Kind::Gate {
            gate: "commit".to_owned(),
            tree: tree.clone(),
            verdict,
        },
    );
    let landed = Event::new(
        Subject::Work(number),
        at,
        Kind::Landed {
            commit: commit.clone(),
            tree,
            evidence: Evidence::Gate,
        },
    );
    let message = message(
        &config::fill(&context.repo.config.subject_landed, area, id),
        &format!("{}\nCommit {}.", work.title, short(&commit)),
        &trailer,
        trailers,
    );
    record_and_commit(&context.repo, vec![gate, landed], &message)
}

fn abandon(id: &str, reason: &str, trailers: &[String]) -> Result<u8, Refusal> {
    if reason.trim().is_empty() {
        return Err(usage(
            code::REASON_REQUIRED,
            "a non-empty reason is required: --reason <reason>",
        ));
    }
    let context = Context::open()?;
    let number = work_number(id)?;
    let work = context.work(id)?;
    let stage = context.journal.stage(&number);
    if stage.is_finished() {
        return Err(refused(
            code::WORK_STAGE,
            format!("work {id} is already {}", stage_text(stage)),
        ));
    }
    let area = work.area(id)?;
    let event = Event::new(
        Subject::Work(number),
        time::now(),
        Kind::Abandoned {
            reason: reason.trim().to_owned(),
        },
    );
    let message = message(
        &config::fill(&context.repo.config.subject_abandoned, area, id),
        &format!("{}\nReason: {}", work.title, reason.trim()),
        &format!("Dacc-Work: {id}"),
        trailers,
    );
    record_and_commit(&context.repo, vec![event], &message)
}

fn state(id: Option<&str>, format: Format) -> Result<u8, Refusal> {
    let context = Context::open()?;
    let selected = id.map(work_number).transpose()?;
    let works = context.works()?;
    if let (Some(id), Some(number)) = (id, selected.as_ref()) {
        if !works.iter().any(|(n, _)| n == number) {
            return Err(refused(
                code::WORK_NOT_IN_PLAN,
                format!(
                    "{id} is not in the plan: no file {}/{id}.rs",
                    context.repo.config.work_dir()
                ),
            ));
        }
    }
    let filtered: Vec<&(String, Record)> = works
        .iter()
        .filter(|(n, _)| selected.as_ref().is_none_or(|selected| selected == n))
        .collect();
    match format {
        Format::Text => {
            for (number, work) in filtered {
                let slice = work.slice.clone().unwrap_or_else(|| "s????".to_owned());
                println!(
                    "{number}  {:<22}  {slice}  {}",
                    stage_text(context.journal.stage(number)),
                    work.title
                );
            }
            if selected.is_none() {
                let closed: Vec<String> = context.journal.closed_slices.keys().cloned().collect();
                if closed.is_empty() {
                    println!("no closed slices");
                } else {
                    println!("closed slices: {}", closed.join(", "));
                }
            }
        }
        Format::Json => {
            let works_json: Vec<String> = filtered
                .into_iter()
                .map(|(number, work)| {
                    let slice = work.slice.clone().unwrap_or_else(|| "s????".to_owned());
                    format!(
                        "{{\"id\":{},\"state\":{},\"slice\":{},\"title\":{}}}",
                        format::string(number),
                        format::string(stage_text(context.journal.stage(number))),
                        format::string(&slice),
                        format::string(&work.title)
                    )
                })
                .collect();
            let mut out = format!("{{\"works\":[{}]", works_json.join(","));
            if selected.is_none() {
                let closed: Vec<String> = context.journal.closed_slices.keys().cloned().collect();
                out.push_str(",\"closed_slices\":[");
                out.push_str(
                    &closed
                        .iter()
                        .map(|slice| format::string(slice))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                out.push(']');
            }
            out.push('}');
            println!("{out}");
        }
    }
    Ok(0)
}

fn close(id: &str, trailers: &[String]) -> Result<u8, Refusal> {
    let number = match Subject::parse(id) {
        Some(Subject::Slice(number)) => number,
        _ => {
            return Err(usage(
                code::SLICE_ID,
                format!("{id} is not a slice id of the form s-versioning"),
            ))
        }
    };
    let context = Context::open()?;
    let slice = context.slice(id)?;
    if let Some(file) = context.journal.closed_slices.get(&number) {
        return Err(refused(
            code::SLICE_ALREADY_CLOSED,
            format!("slice {id} is already closed by event {file}"),
        ));
    }
    let works: Vec<(String, Record)> = context
        .works()?
        .into_iter()
        .filter(|(_, work)| work.slice.as_deref() == Some(number.as_str()))
        .collect();
    let Some((first, first_work)) = works.first() else {
        return Err(refused(
            code::SLICE_NO_WORKS,
            format!("slice {id} has no works"),
        ));
    };
    let unfinished: Vec<String> = works
        .iter()
        .filter(|(n, _)| !context.journal.stage(n).is_finished())
        .map(|(n, _)| n.clone())
        .collect();
    if !unfinished.is_empty() {
        return Err(refused(
            code::SLICE_UNFINISHED,
            format!(
                "slice {id} is not closed: {} not finished",
                unfinished.join(", ")
            ),
        ));
    }
    let area = first_work.area(first)?;
    let event = Event::new(Subject::Slice(number), time::now(), Kind::Closed);
    let message = message(
        &config::fill(&context.repo.config.subject_slice_closed, area, id),
        &slice.title,
        &format!("Dacc-Slice: {id}"),
        trailers,
    );
    record_and_commit(&context.repo, vec![event], &message)
}

/// Репозиторий и свёртка журнала рабочего дерева.
pub struct Context {
    pub repo: git::Repo,
    journal: Journal,
}

impl Context {
    /// Открывает репозиторий и сворачивает журнал. Несворачиваемый журнал —
    /// отказ: новое событие поверх нарушения ничего не прояснит.
    pub fn open() -> Result<Context, Refusal> {
        let repo = git::Repo::discover(Path::new("."))
            .map_err(|problem| usage(code::REPO_DISCOVER, problem))?;
        let dir = repo.root.join(repo.config.journal_dir());
        let (events, read_violations) = dacc_journal::read_dir(&dir).map_err(|error| {
            usage(
                code::JOURNAL_NOT_READ,
                format!("journal {} not read: {error}", dir.display()),
            )
        })?;
        let (journal, fold_violations) = fold(&events);
        if let Some(violation) = read_violations.iter().chain(&fold_violations).next() {
            return Err(refused(
                code::JOURNAL_NOT_FOLD,
                format!(
                    "journal does not fold: {}: {}",
                    violation.file, violation.reason
                ),
            ));
        }
        Ok(Context { repo, journal })
    }

    /// Запись работы из плана.
    fn work(&self, id: &str) -> Result<Record, Refusal> {
        self.record(&self.repo.config.work_dir(), id)
    }

    /// Запись среза из плана.
    fn slice(&self, id: &str) -> Result<Record, Refusal> {
        self.record(&self.repo.config.slice_dir(), id)
    }

    /// Запись плана `<каталог>/<id>.rs`.
    fn record(&self, dir: &str, id: &str) -> Result<Record, Refusal> {
        let path = self.repo.root.join(dir).join(format!("{id}.rs"));
        fs::read_to_string(&path)
            .map(|text| Record::parse(&text))
            .map_err(|_| {
                refused(
                    code::WORK_NOT_IN_PLAN,
                    format!("{id} is not in the plan: no file {dir}/{id}.rs"),
                )
            })
    }

    /// Все работы плана по порядку номеров.
    fn works(&self) -> Result<Vec<(String, Record)>, Refusal> {
        let dir = self.repo.root.join(self.repo.config.work_dir());
        let entries = fs::read_dir(&dir).map_err(|error| {
            usage(
                code::PLAN_NOT_READ,
                format!("plan {} not read: {error}", dir.display()),
            )
        })?;
        let mut works = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(Subject::Work(number)) = name.strip_suffix(".rs").and_then(Subject::parse)
            else {
                continue;
            };
            let text = fs::read_to_string(entry.path()).unwrap_or_default();
            works.push((number, Record::parse(&text)));
        }
        works.sort_by_key(|(number, _)| number.clone());
        Ok(works)
    }
}

/// То, что команды читают из текста файла работы или среза.
struct Record {
    title: String,
    slice: Option<String>,
    area: Option<String>,
}

impl Record {
    fn parse(text: &str) -> Record {
        const SLICE: &str = "slice: crate::slice::s";
        const TAXON: &str = "taxon!(Subsystem, ";
        Record {
            title: quoted_after(text, "title: NonEmptyStr::new(").unwrap_or_default(),
            slice: text
                .find(SLICE)
                .and_then(|at| text.get(at + SLICE.len()..))
                .and_then(|rest| rest.split([',', ')']).next())
                .filter(|slug| !slug.is_empty())
                .map(|slug| format!("s{}", slug.replace('_', "-"))),
            area: text
                .find(TAXON)
                .and_then(|at| text[at + TAXON.len()..].split_once(')'))
                .map(|(name, _)| name.trim().to_lowercase()),
        }
    }

    /// Область темы коммита — подсистема работы.
    fn area(&self, id: &str) -> Result<&str, Refusal> {
        self.area.as_deref().ok_or_else(|| {
            refused(
                code::SUBSYSTEM_NOT_READ,
                format!("the subsystem taxon!(Subsystem, …) of {id} cannot be read"),
            )
        })
    }
}

/// `cargo dacc journal import --work <w-slug> [--close-finished-slices]`.
pub fn run_import(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, trailers)| {
        match words
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["--work", basis] => import(basis, false, &trailers),
            ["--work", basis, "--close-finished-slices"]
            | ["--close-finished-slices", "--work", basis] => import(basis, true, &trailers),
            _ => Err(usage(
                code::USAGE,
                "journal import --work <w-slug> [--close-finished-slices]",
            )),
        }
    }))
}

/// Приземления из истории (решение 15): каждая запланированная работа, у
/// которой в истории HEAD есть коммит с её трейлером, приземляется по
/// последнему такому коммиту. Работа без коммита остаётся запланированной —
/// импорт не угадывает. Основание импорта пропускается: оно ещё в работе.
/// С `close_slices` закрываются срезы, все работы которых после импорта
/// завершены.
fn import(basis: &str, close_slices: bool, trailers: &[String]) -> Result<u8, Refusal> {
    let context = Context::open()?;
    let basis_number = work_number(basis)?;
    let area = context.work(basis)?.area(basis)?.to_owned();
    let root = &context.repo.root;

    // История идёт от новых коммитов к старым: первый встреченный — последний.
    // Читаются оба трейлера: новый Dacc-Work и прежний Slipway-Work (до
    // переименования, решение 29), чтобы импорт видел старую историю.
    let log = git::read(
        root,
        &[
            "log",
            "--format=%H %(trailers:key=Dacc-Work,valueonly,separator=%x20) %(trailers:key=Slipway-Work,valueonly,separator=%x20)",
            "HEAD",
        ],
    )
    .unwrap_or_default();
    let mut latest = std::collections::BTreeMap::new();
    for line in log.lines() {
        let mut fields = line.split_whitespace();
        let Some(commit) = fields.next() else {
            continue;
        };
        for id in fields {
            if let Some(Subject::Work(number)) = Subject::parse(id) {
                latest.entry(number).or_insert_with(|| commit.to_owned());
            }
        }
    }

    let at = time::now();
    let works = context.works()?;
    let mut events = Vec::new();
    let mut imported = std::collections::BTreeSet::new();
    for (number, _) in &works {
        if number.as_str() == basis_number.as_str()
            || context.journal.stage(number) != Stage::Planned
        {
            continue;
        }
        let Some(commit) = latest.get(number) else {
            continue;
        };
        let tree = proof::content_hash(root, &context.repo.config.journal_dir(), commit)
            .ok_or_else(|| {
                usage(
                    code::TREE_NOT_READ,
                    format!("the tree of commit {} cannot be read", short(commit)),
                )
            })?;
        events.push(Event::new(
            Subject::Work(number.clone()),
            at.clone(),
            Kind::Landed {
                commit: commit.clone(),
                tree,
                evidence: Evidence::History,
            },
        ));
        imported.insert(number.clone());
    }

    let mut closed = Vec::new();
    if close_slices {
        let mut finished = std::collections::BTreeMap::new();
        for (number, work) in &works {
            let Some(slice) = &work.slice else {
                continue;
            };
            let done = context.journal.stage(number).is_finished() || imported.contains(number);
            let all = finished.entry(slice.clone()).or_insert(true);
            *all = *all && done;
        }
        for (slice, done) in finished {
            if done && !context.journal.closed_slices.contains_key(&slice) {
                closed.push(slice.clone());
                events.push(Event::new(Subject::Slice(slice), at.clone(), Kind::Closed));
            }
        }
    }

    if events.is_empty() {
        return Err(refused(
            code::NOTHING_TO_IMPORT,
            "nothing to import: planned works have no commits carrying their trailer",
        ));
    }
    let imported: Vec<String> = imported.into_iter().collect();
    let body = format!(
        "Landed from history: {}.\nClosed slices: {}.",
        listed(&imported),
        listed(&closed)
    );
    let message = message(
        &config::fill(&context.repo.config.subject_imported, &area, basis),
        &body,
        &format!("Dacc-Work: {basis}"),
        trailers,
    );
    record_and_commit(&context.repo, events, &message)
}

fn listed(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}

/// Строка в кавычках после `marker`; между маркером и кавычкой допустимы
/// пробелы и переводы строк.
fn quoted_after(text: &str, marker: &str) -> Option<String> {
    let rest = text[text.find(marker)? + marker.len()..]
        .trim_start()
        .strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push(chars.next()?),
            '"' => return Some(out),
            c => out.push(c),
        }
    }
    None
}

/// Номер работы из `w-slug`.
fn work_number(id: &str) -> Result<String, Refusal> {
    match Subject::parse(id) {
        Some(Subject::Work(number)) => Ok(number),
        _ => Err(usage(
            code::NOT_WORK_ID,
            format!("{id} is not a work id of the form w-fix-gitignore"),
        )),
    }
}

fn stage_text(stage: Stage) -> &'static str {
    match stage {
        Stage::Planned => "planned",
        Stage::Started => "started",
        Stage::Landed => "landed",
        Stage::LandedFromHistory => "landed from history",
        Stage::Abandoned => "abandoned",
    }
}

fn short(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

/// Вердикт калитки для события gate из доказательства: структурный, если
/// доказательство новое, иначе проза прежнего доказательства (решение 22).
fn gate_verdict(common_dir: &Path, tree: &str) -> Option<GateVerdict> {
    if let Some(json) = proof::structured(common_dir, tree) {
        return structured_verdict(&json);
    }
    proof::verdict(common_dir, tree).map(GateVerdict::Prose)
}

/// Структурный вердикт из JSON доказательства: поля пройденных шагов, их
/// общего числа, пропущенных, числа прошедших атак и минимального тулчейна.
fn structured_verdict(json: &str) -> Option<GateVerdict> {
    let field = |key: &str| {
        let needle = format!("\"{key}\":");
        let rest = json.find(&needle).map(|at| &json[at + needle.len()..])?;
        if let Some(stripped) = rest.strip_prefix('"') {
            let end = stripped.find('"')?;
            Some(stripped[..end].to_owned())
        } else {
            let end = rest.find([',', '}']).unwrap_or(rest.len());
            Some(rest[..end].trim().to_owned())
        }
    };
    let number = |key: &str| field(key).and_then(|value| value.parse::<usize>().ok());
    let skipped = json
        .find("\"skipped\":[")
        .and_then(|at| {
            let rest = &json[at + "\"skipped\":[".len()..];
            let end = rest.find(']')?;
            Some(&rest[..end])
        })
        .map(|inner| {
            inner
                .split(',')
                .map(|item| item.trim().trim_matches('"').to_owned())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Some(GateVerdict::Structured {
        passed: number("passed")?,
        total: number("total")?,
        skipped,
        attacks: number("attacks")?,
        msrv: field("msrv")?,
    })
}

/// Сообщение коммита события: тема, тело, трейлер основания и добавленные
/// трейлеры.
fn message(subject: &str, body: &str, basis: &str, trailers: &[String]) -> String {
    let mut text = format!("{subject}\n\n{body}\n\n{basis}\n");
    for trailer in trailers {
        text.push_str(trailer);
        text.push('\n');
    }
    text
}

/// Записывает события в журнал и коммитит ровно их. Если коммит не создан,
/// записанные файлы удаляются: событие без коммита — не история.
fn record_and_commit(repo: &git::Repo, events: Vec<Event>, message: &str) -> Result<u8, Refusal> {
    let journal = repo.root.join(repo.config.journal_dir());
    let mut written: Vec<PathBuf> = Vec::new();
    for event in events {
        let mut attempt = 0;
        let mut relative = event.file.clone();
        while journal.join(&relative).exists() {
            attempt += 1;
            relative = relative_path(&event.subject, &event.at, &event.kind, attempt);
        }
        let path = journal.join(&relative);
        let text = Event {
            file: relative,
            ..event
        }
        .to_text();
        let stored = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(&path, text));
        if let Err(error) = stored {
            remove(&written);
            return Err(usage(
                code::EVENT_NOT_WRITTEN,
                format!("event {} not written: {error}", path.display()),
            ));
        }
        written.push(path);
    }
    let message_file = repo.git_dir.join(layout::JOURNAL_MESSAGE);
    if let Err(error) = fs::write(&message_file, message) {
        remove(&written);
        return Err(usage(
            code::COMMIT_MESSAGE_NOT_WRITTEN,
            format!("commit message not written: {error}"),
        ));
    }
    let mut args = vec![
        OsString::from("-F"),
        message_file.clone().into_os_string(),
        OsString::from("--"),
    ];
    for path in &written {
        args.push(
            path.strip_prefix(&repo.root)
                .unwrap_or(path)
                .as_os_str()
                .to_owned(),
        );
    }
    let code = commit::run(&args);
    let _ = fs::remove_file(&message_file);
    if code != 0 {
        remove(&written);
    }
    Ok(code)
}

fn remove(files: &[PathBuf]) {
    for file in files {
        let _ = fs::remove_file(file);
    }
}
