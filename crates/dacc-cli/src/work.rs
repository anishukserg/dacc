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
use crate::record::{
    anchor_list_is_empty, field_slug, field_string, field_value, flat, last_ident, record_fields,
    string_literal,
};
use crate::{commit, config, gate, git, hooks, layout, proof};
use dacc_journal::{
    fold, time, Event, Evidence, GateVerdict, Journal, Kind, Proof, Stage, Subject,
};
use std::ffi::OsString;
use std::fs;
use std::path::Path;

const WORK_USAGE: &str = "work start <w-slug> | new <w-slug> --slice … --origin … | next [--format json] | land <w-slug> [--commit <revision>] [--red-before <subject>] [--mutation-proof <subject>] [--anti-vacuum <subject>] | drop <w-slug> --reason <reason> | state [<w-slug>]";

const SLICE_USAGE: &str = "slice close <s-slug>";

/// `cargo dacc work …`.
pub fn run_work(args: &[OsString]) -> u8 {
    let format = format::scan(args);
    let outcome = words(args).and_then(|(words, trailers)| {
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
            ["next"] => crate::work_next::next(format),
            ["land", id, rest @ ..] => land(id, rest, &trailers),
            ["drop", id, "--reason", reason] => abandon(id, reason, &trailers),
            ["state"] => state(None, format),
            ["state", id] => state(Some(id), format),
            ["new", tail @ ..] => {
                let args: Vec<String> = tail.iter().map(|word| (*word).to_owned()).collect();
                crate::work_new::work_new(&args)
            }
            _ => Err(usage(code::USAGE, WORK_USAGE)),
        }
    });
    crate::access::finish_json(format, outcome)
}

/// `cargo dacc slice …`.
pub fn run_slice(args: &[OsString]) -> u8 {
    let format = format::scan(args);
    let outcome = words(args).and_then(|(words, trailers)| {
        match words
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["close", id] => close(id, &trailers),
            _ => Err(usage(code::USAGE, SLICE_USAGE)),
        }
    });
    crate::access::finish_json(format, outcome)
}

/// `cargo dacc metrics` (решение 44): счётчики пилота одной командой — коммиты
/// с основанием, записи журнала по видам, радиусы и стадии работ.
pub fn run_metrics(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, _)| {
        if !words.is_empty() {
            return Err(usage(code::USAGE, "metrics"));
        }
        metrics()
    }))
}

/// Считает счётчики из реестра и журнала и печатает их построчно.
fn metrics() -> Result<u8, Refusal> {
    let context = Context::open()?;
    let root = &context.repo.root;

    // Коммиты с основанием: трейлер Dacc-Work / Dacc-Slice в тексте коммита.
    // Считаются коммиты, а не строки трейлеров: коммит с двумя трейлерами
    // несёт одно основание (работа w-access-answers).
    let total = git::read(root, &["rev-list", "--count", "HEAD"])
        .and_then(|out| out.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let with_basis = git::read(root, &["log", "--format=%x1e%H%n%B"])
        .map(|log| basis_commits(&log))
        .unwrap_or(0);

    // Стадии работ и радиусы из плана.
    let works = context.works()?;
    let mut stages = [0usize; 5];
    let mut radii = std::collections::BTreeMap::<String, usize>::new();
    for (number, work) in &works {
        stages[stage_index(context.journal.stage(number))] += 1;
        if let Some(radius) = &work.radius {
            *radii.entry(radius.clone()).or_default() += 1;
        }
    }

    // Записи журнала по видам: одна запись и прежние файлы-события.
    let file = root.join(context.repo.config.journal_file());
    let dir = root.join(context.repo.config.journal_dir());
    let (file_events, _) =
        dacc_journal::read_file(&file).unwrap_or_else(|_| (Vec::new(), Vec::new()));
    let (dir_events, _) = if dir.is_dir() {
        dacc_journal::read_dir(&dir).unwrap_or_else(|_| (Vec::new(), Vec::new()))
    } else {
        (Vec::new(), Vec::new())
    };
    let mut kinds = [0usize; 5];
    for event in file_events.iter().chain(&dir_events) {
        kinds[kind_index(&event.kind)] += 1;
    }

    println!("commits: {total} total, {with_basis} with basis");
    println!(
        "work states: {} planned, {} started, {} landed, {} from history, {} abandoned",
        stages[0], stages[1], stages[2], stages[3], stages[4]
    );
    let radii_text = if radii.is_empty() {
        "none".to_owned()
    } else {
        radii
            .iter()
            .map(|(radius, count)| format!("{radius} {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("radii: {radii_text}");
    println!(
        "journal: {} started, {} gate, {} landed, {} abandoned, {} closed",
        kinds[0], kinds[1], kinds[2], kinds[3], kinds[4]
    );
    Ok(0)
}

/// Число коммитов, несущих основание: запись с трейлером Dacc-Work или
/// Dacc-Slice. Коммит с двумя трейлерами считается один раз.
fn basis_commits(log: &str) -> usize {
    log.split('\x1e')
        .filter(|record| {
            record
                .lines()
                .any(|line| line.starts_with("Dacc-Work:") || line.starts_with("Dacc-Slice:"))
        })
        .count()
}

/// Индекс стадии в счётчике работ: planned, started, landed, from history,
/// abandoned.
fn stage_index(stage: Stage) -> usize {
    match stage {
        Stage::Planned => 0,
        Stage::Started => 1,
        Stage::Landed => 2,
        Stage::LandedFromHistory => 3,
        Stage::Abandoned => 4,
    }
}

/// Индекс вида в счётчике журнала: started, gate, landed, abandoned, closed.
fn kind_index(kind: &Kind) -> usize {
    match kind {
        Kind::Started => 0,
        Kind::Gate { .. } => 1,
        Kind::Landed { .. } => 2,
        Kind::Abandoned { .. } => 3,
        Kind::Closed => 4,
    }
}

/// `cargo dacc upgrade [<from>] <to>` (решение 44): шаги повышения версии из
/// реестра upgrade!, сгруппированные по версии. Без аргументов — все шаги.
pub fn run_upgrade(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, _)| {
        let (from, to) = match words.as_slice() {
            [] => (None, None),
            [to] => (None, Some(to.as_str())),
            [from, to] => (Some(from.as_str()), Some(to.as_str())),
            _ => return Err(usage(code::USAGE, "upgrade [<from>] <to>")),
        };
        upgrade(from, to)
    }))
}

/// Читает реестр upgrade! и печатает шаги subject + how по версиям.
fn upgrade(from: Option<&str>, to: Option<&str>) -> Result<u8, Refusal> {
    let context = Context::open()?;
    let dir = context.repo.root.join(context.repo.config.upgrade_dir());
    let records = read_upgrades(&dir)?;
    let mut grouped = std::collections::BTreeMap::<&str, Vec<&UpgradeRecord>>::new();
    for record in &records {
        if to.is_some_and(|to| record.to.as_str() != to)
            || from.is_some_and(|from| record.from.as_str() != from)
        {
            continue;
        }
        grouped.entry(record.to.as_str()).or_default().push(record);
    }
    if grouped.is_empty() {
        let target = to.map_or("the registry".to_owned(), str::to_string);
        println!("no upgrade steps for {target}");
        return Ok(0);
    }
    for (version, steps) in grouped {
        println!("Upgrade to {version}:");
        for step in steps {
            println!("  {}: {}", step.subject, step.how);
        }
    }
    Ok(0)
}

/// Запись инструкции повышения версии, прочитанная из текста файла `u-*.rs`.
struct UpgradeRecord {
    from: String,
    to: String,
    subject: String,
    how: String,
}

impl UpgradeRecord {
    fn parse(text: &str) -> Option<UpgradeRecord> {
        Some(UpgradeRecord {
            from: field_string(text, "from")?,
            to: field_string(text, "to")?,
            subject: field_string(text, "subject")?,
            how: field_string(text, "how")?,
        })
    }
}

/// Читает записи upgrade! из каталога; отсутствующий каталог — пустой реестр.
fn read_upgrades(dir: &Path) -> Result<Vec<UpgradeRecord>, Refusal> {
    let mut records = Vec::new();
    if !dir.is_dir() {
        return Ok(records);
    }
    let entries = fs::read_dir(dir).map_err(|error| {
        usage(
            code::PLAN_NOT_READ,
            format!("upgrade {} not read: {error}", dir.display()),
        )
    })?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".rs") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        if let Some(record) = UpgradeRecord::parse(&text) {
            records.push(record);
        }
    }
    records.sort_by(|a, b| a.to.cmp(&b.to).then_with(|| a.subject.cmp(&b.subject)));
    Ok(records)
}

/// Отказ команды: код возврата, стабильный код причины и текст.
#[derive(Debug)]
pub struct Refusal {
    exit_code: u8,
    refusal: Code,
    reason: String,
}

impl Refusal {
    /// Код причины отказа — машинный контракт (решение 22).
    pub(crate) fn code(&self) -> &str {
        self.refusal
    }

    /// Причина отказа человеческим текстом.
    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }

    /// Код возврата процесса.
    pub(crate) fn exit_code(&self) -> u8 {
        self.exit_code
    }
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

pub(crate) fn finish(outcome: Result<u8, Refusal>) -> u8 {
    match outcome {
        Ok(code) => code,
        Err(refusal) => {
            println!("WORK REFUSED: {} [{}]", refusal.reason, refusal.refusal);
            refusal.exit_code
        }
    }
}

/// Аргументы словами и строки `--trailer`, извлечённые из любого места.
pub(crate) fn words(args: &[OsString]) -> Result<(Vec<String>, Vec<String>), Refusal> {
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

fn start(id: &str, _trailers: &[String]) -> Result<u8, Refusal> {
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
    // Решение 42: событие started пишется командой work land вместе с gate и
    // landed, а не отдельным коммитом. Команда start только проверяет, что
    // работа запланирована и её срез открыт.
    println!("work {id} is planned; it will be recorded when it lands");
    Ok(0)
}

fn land(id: &str, flags: &[&str], trailers: &[String]) -> Result<u8, Refusal> {
    let mut revision = "HEAD".to_owned();
    let mut proofs: Vec<Proof> = Vec::new();
    let mut words = flags.iter();
    while let Some(&flag) = words.next() {
        let value = words
            .next()
            .copied()
            .ok_or_else(|| usage(code::FLAG_NEEDS_VALUE, format!("{flag} needs a value")))?;
        match flag {
            "--commit" => revision = value.to_owned(),
            "--red-before" | "--mutation-proof" | "--anti-vacuum" => {
                let build: fn(String) -> Proof = match flag {
                    "--red-before" => Proof::RedBefore,
                    "--mutation-proof" => Proof::MutationProof,
                    _ => Proof::AntiVacuum,
                };
                let subject = value.trim().to_owned();
                if subject.is_empty() {
                    return Err(usage(
                        code::PROOF_SUBJECT_REQUIRED,
                        format!("{flag} needs a non-empty subject"),
                    ));
                }
                let kind = build(subject.clone()).key();
                if proofs.iter().any(|proof| proof.key() == kind) {
                    return Err(usage(
                        code::PROOF_REPEATED,
                        format!("{flag} is stated once"),
                    ));
                }
                proofs.push(build(subject));
            }
            other => {
                return Err(usage(
                    code::UNKNOWN_ARGUMENT,
                    format!("unknown argument {other}"),
                ))
            }
        }
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
    if let Some(slice) = &work.slice {
        if let Some(file) = context.journal.closed_slices.get(slice) {
            return Err(refused(
                code::SLICE_ALREADY_CLOSED,
                format!("slice {slice} of work {id} is closed by event {file}"),
            ));
        }
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
    // Гарантия «тест падал до починки» получает исполнителя (работа
    // w-evidence-land): дефект — происхождение Divergence — или необратимое
    // изменение не приземляются без вида доказательства RedBefore.
    let needs_red_before =
        work.divergence || matches!(work.radius.as_deref(), Some("persistent" | "irreversible"));
    if needs_red_before
        && !proofs
            .iter()
            .any(|proof| matches!(proof, Proof::RedBefore(_)))
    {
        return Err(refused(
            code::PROOF_RED_BEFORE,
            format!(
                "work {id} is a defect fix or an irreversible change and lands without a RedBefore proof: name the test that was red before the fix with --red-before <subject>"
            ),
        ));
    }
    // RedBefore называет существующий тест дерева (работа w-red-before-exists):
    // вымышленное имя отвергается, а калитка исполняет названные тесты дерева.
    for proof in &proofs {
        if let Proof::RedBefore(subject) = proof {
            let name = subject.rsplit("::").next().unwrap_or(subject);
            let found = git::read(
                &context.repo.root,
                &[
                    "grep",
                    "-l",
                    "-e",
                    &format!("fn {name}"),
                    &commit,
                    "--",
                    "*.rs",
                ],
            )
            .is_some_and(|hits| !hits.trim().is_empty());
            if !found {
                return Err(refused(
                    code::RED_BEFORE_UNKNOWN,
                    format!("RedBefore names no test of the commit tree: {subject}"),
                ));
            }
        }
    }
    let journal = context.repo.config.journal_dir();
    let tree = proof::content_hash(root, &journal, &commit).ok_or_else(|| {
        usage(
            code::TREE_NOT_READ,
            format!("the tree of commit {} cannot be read", short(&commit)),
        )
    })?;
    let verdict = full_gate_verdict(&context.repo, &commit).map_err(|problem| {
        refused(
            code::FULL_TIER_FAILED,
            format!(
                "the full gate tier did not pass on commit {}: {problem}",
                short(&commit)
            ),
        )
    })?;
    let area = work.area(id)?;
    let at = time::now();
    // Решение 42: started пишется здесь же, вместе с gate и landed — церемония
    // перестаёт быть отдельным коммитом за событие started. Работа, начатая
    // прежним work start, уже несёт started, и второй не пишется.
    let mut events = Vec::new();
    if stage == Stage::Planned {
        events.push(Event::new(
            Subject::Work(number.clone()),
            at.clone(),
            Kind::Started,
        ));
    }
    events.push(Event::new(
        Subject::Work(number.clone()),
        at.clone(),
        Kind::Gate {
            gate: "commit".to_owned(),
            tree: tree.clone(),
            verdict,
        },
    ));
    events.push(Event::new(
        Subject::Work(number),
        at,
        Kind::Landed {
            commit: commit.clone(),
            tree,
            evidence: Evidence::Gate,
            proofs,
        },
    ));
    let message = message(
        &config::fill(&context.repo.config.subject_landed, area, id),
        &format!("{}\nCommit {}.", work.title, short(&commit)),
        &trailer,
        trailers,
    );
    record_and_commit(&context.repo, events, &message)
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
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
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
    check_contract(&context, id, &slice)?;
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

/// Контракт спецификации имеет исполнителя (работа w-contract-close-gate):
/// закрытие среза требует записи инварианта своей спецификации со статусом
/// Enforced и живым якорем enforced_by. Живость якоря — путь к константе,
/// порождённой сканом: неживой якорь не компилируется, а пустой список здесь
/// отвергается как контракт без исполнителя.
fn check_contract(context: &Context, id: &str, slice: &Record) -> Result<(), Refusal> {
    let spec = slice.specification.as_deref().ok_or_else(|| {
        refused(
            code::CONTRACT_UNENFORCED,
            format!("slice {id} has no specification: the contract has no executor"),
        )
    })?;
    let dir = context
        .repo
        .root
        .join(format!("{}/invariant", context.repo.config.doc));
    let mut found = 0;
    for (record_id, text) in crate::access::records(&dir) {
        let fields = record_fields(&text);
        // Спецификация сравнивается идентификатором: форма пути и переносы не
        // участвуют в сравнении.
        let linked = field_value(&fields, "specification")
            .and_then(last_ident)
            .is_some_and(|linked| linked.replace('_', "-") == spec.replace('_', "-"));
        if !linked {
            continue;
        }
        found += 1;
        let status = field_value(&fields, "status").unwrap_or_default();
        if !flat(status).starts_with("InvariantStatus::Enforced") {
            return Err(refused(
                code::CONTRACT_UNENFORCED,
                format!("invariant {record_id} of {spec} is not Enforced"),
            ));
        }
        // Пустой якорь ловится в любой форме записи: комментарии сняты, пробелы
        // схлопнуты, значение прочитано сбалансированной группой.
        if anchor_list_is_empty(field_value(&fields, "enforced_by").unwrap_or_default()) {
            return Err(refused(
                code::CONTRACT_UNENFORCED,
                format!("invariant {record_id} of {spec} has no live anchor"),
            ));
        }
        // Enforced без тестового якоря — заявление, а не доказательство
        // (работа w-enforced-needs-tests): статус требует якоря на исполняемый
        // тест, который калитка гоняет на каждом дереве.
        if anchor_list_is_empty(field_value(&fields, "tests").unwrap_or_default()) {
            return Err(refused(
                code::CONTRACT_UNENFORCED,
                format!("invariant {record_id} of {spec} has no test anchor"),
            ));
        }
    }
    if found == 0 {
        return Err(refused(
            code::CONTRACT_UNENFORCED,
            format!("specification {spec} has no invariant record: the contract has no executor"),
        ));
    }
    Ok(())
}

/// Репозиторий и свёртка журнала рабочего дерева.
pub struct Context {
    pub repo: git::Repo,
    pub(crate) journal: Journal,
}

impl Context {
    /// Открывает репозиторий и сворачивает журнал. Несворачиваемый журнал —
    /// отказ: новое событие поверх нарушения ничего не прояснит.
    pub fn open() -> Result<Context, Refusal> {
        let repo = git::Repo::discover(Path::new("."))
            .map_err(|problem| usage(code::REPO_DISCOVER, problem))?;
        let file = repo.root.join(repo.config.journal_file());
        let dir = repo.root.join(repo.config.journal_dir());
        let (mut events, mut read_violations) =
            dacc_journal::read_file(&file).map_err(|error| {
                usage(
                    code::JOURNAL_NOT_READ,
                    format!("journal {} not read: {error}", file.display()),
                )
            })?;
        // Прежние файлы — замороженная история (решение 43); их может не быть у
        // нового проекта.
        if dir.is_dir() {
            let (dir_events, dir_violations) = dacc_journal::read_dir(&dir).map_err(|error| {
                usage(
                    code::JOURNAL_NOT_READ,
                    format!("journal {} not read: {error}", dir.display()),
                )
            })?;
            events.extend(dir_events);
            read_violations.extend(dir_violations);
        }
        let (journal, fold_violations) = fold(&events, &[], &[]);
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
    pub(crate) fn works(&self) -> Result<Vec<(String, Record)>, Refusal> {
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

// Разбор записей реестра — в модуле `crate::record`: значениями полей, в
// одном месте на инструмент (работа w-access-split).

/// То, что команды читают из текста файла работы или среза.
pub(crate) struct Record {
    pub(crate) title: String,
    pub(crate) slice: Option<String>,
    pub(crate) area: Option<String>,
    pub(crate) radius: Option<String>,
    /// Происхождение Divergence — расхождение есть дефект: его починка без
    /// RedBefore не приземляется.
    pub(crate) divergence: bool,
    /// Спецификация записи: `specification: crate::rfc::X`. У среза — это
    /// спецификация, чей контракт проверяется на закрытии.
    pub(crate) specification: Option<String>,
}

impl Record {
    pub(crate) fn parse(text: &str) -> Record {
        let fields = record_fields(text);
        Record {
            title: field_value(&fields, "title")
                .and_then(string_literal)
                .unwrap_or_default(),
            slice: field_slug(text, "slice"),
            area: field_value(&fields, "taxon").map(|value| {
                value
                    .rsplit_once(',')
                    .map_or(value, |(_, name)| name)
                    .trim()
                    .trim_end_matches(')')
                    .trim()
                    .to_lowercase()
            }),
            radius: field_value(&fields, "radius")
                .and_then(last_ident)
                .map(|radius| radius.to_lowercase()),
            divergence: field_value(&fields, "origin")
                .is_some_and(|value| flat(value).contains("Divergence")),
            specification: field_value(&fields, "specification").and_then(last_ident),
        }
    }

    /// Область темы коммита — подсистема работы.
    pub(crate) fn area(&self, id: &str) -> Result<&str, Refusal> {
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
                proofs: Vec::new(),
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

/// Номер работы из `w-slug`.
pub(crate) fn work_number(id: &str) -> Result<String, Refusal> {
    match Subject::parse(id) {
        Some(Subject::Work(number)) => Ok(number),
        _ => Err(usage(
            code::NOT_WORK_ID,
            format!("{id} is not a work id of the form w-fix-gitignore"),
        )),
    }
}

/// Тексты стадий в порядке автомата — единственный источник текста стадии:
/// ответы слоя доступа и счётчики перечисляют стадии отсюда, и пропущенная
/// стадия видна тестом (работа w-access-answers).
pub(crate) const STAGE_TEXTS: [&str; 5] = [
    "planned",
    "started",
    "landed",
    "landed from history",
    "abandoned",
];

pub(crate) fn stage_text(stage: Stage) -> &'static str {
    STAGE_TEXTS[stage_index(stage)]
}

fn short(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

/// Полный ярус закрытия работы (решение 33): выгружает дерево коммита и
/// исполняет `gate --full` — ярус коммита плюс MSRV и зависимости. Вердикт
/// структурный и обязательно с `msrv`: ярус коммита работу не приземляет.
fn full_gate_verdict(repo: &git::Repo, commit: &str) -> Result<GateVerdict, String> {
    let tree = repo.git_dir.join(layout::COMMIT_TREE_DIR).join("land");
    hooks::export_commit(repo, commit, &tree)?;
    let args = vec![
        OsString::from("--repo"),
        repo.root.clone().into_os_string(),
        OsString::from("--full"),
        tree.as_os_str().to_owned(),
    ];
    let (code, prose, structured) = gate::run_for_proof(&args);
    if code != 0 {
        return Err(prose);
    }
    let verdict = structured.ok_or_else(|| "the gate returned no structured verdict".to_owned())?;
    let Some(msrv) = verdict.msrv.clone() else {
        return Err("the gate ran the commit tier, not the full tier — no msrv".to_owned());
    };
    Ok(GateVerdict::Structured {
        passed: verdict.passed,
        total: verdict.total,
        skipped: verdict
            .skipped
            .iter()
            .map(|step| step.to_string())
            .collect(),
        attacks: verdict.attacks.unwrap_or(0),
        msrv: Some(msrv),
    })
}

/// Сообщение коммита события: тема, тело, трейлер основания и добавленные
/// трейлеры.
pub(crate) fn message(subject: &str, body: &str, basis: &str, trailers: &[String]) -> String {
    let mut text = format!("{subject}\n\n{body}\n\n{basis}\n");
    for trailer in trailers {
        text.push_str(trailer);
        text.push('\n');
    }
    text
}

/// Сколько секунд запись журнала ждёт чужой лок, прежде чем отказывать:
/// параллельный агент получает короткое ожидание, а не мгновенный отказ
/// (работа w-journal-transaction).
const JOURNAL_LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// Дописывает события в одну запись журнала `journal.toml` и коммитит ровно её
/// (решение 43). Решение о событиях принимается под межпроцессным локом по
/// свежей свёртке: параллельный процесс получает отказ с кодом причины, а не
/// теряет событие в гонке за журнал (работы w-journal-lock и w-work-next).
pub(crate) fn record_and_commit_under_lock(
    repo: &git::Repo,
    decide: impl FnOnce() -> Result<(Vec<Event>, String), Refusal>,
) -> Result<u8, Refusal> {
    let _lock =
        crate::commit::Lock::acquire(&repo.git_dir.join(layout::JOURNAL_LOCK), JOURNAL_LOCK_WAIT)
            .map_err(|problem| {
            refused(
                code::LOCK_NOT_ACQUIRED,
                format!("journal write is locked: {problem}"),
            )
        })?;
    let (events, message) = decide()?;
    let file = repo.root.join(repo.config.journal_file());
    // Событие сверяется со свёрткой под локом: переход поверх нарушения или
    // гонки двух процессов отвергается до записи.
    let mut all = journal_events(repo);
    let start = all.len();
    all.extend(events);
    let (_, violations) = fold(&all, &[], &[]);
    if let Some(violation) = violations.into_iter().next() {
        return Err(refused(
            code::JOURNAL_NOT_FOLD,
            format!(
                "event does not fit the journal: {}: {}",
                violation.file, violation.reason
            ),
        ));
    }
    // Append-only: прежняя версия записи — префикс новой, поэтому читаем текущую
    // и дописываем события таблицами [[events]].
    let existed = file.exists();
    let previous = fs::read_to_string(&file).unwrap_or_default();
    let mut text = previous.clone();
    for event in &all[start..] {
        text.push_str(&event.to_table());
    }
    if let Err(error) = fs::write(&file, &text) {
        return Err(usage(
            code::EVENT_NOT_WRITTEN,
            format!("{} not written: {error}", file.display()),
        ));
    }
    let message_file = repo.git_dir.join(layout::JOURNAL_MESSAGE);
    if let Err(error) = fs::write(&message_file, &message) {
        rollback(repo, &file, &previous, existed);
        return Err(usage(
            code::COMMIT_MESSAGE_NOT_WRITTEN,
            format!("commit message not written: {error}"),
        ));
    }
    let relative = file.strip_prefix(&repo.root).unwrap_or(&file).to_owned();
    let args = vec![
        OsString::from("-F"),
        message_file.clone().into_os_string(),
        OsString::from("--"),
        relative.as_os_str().to_owned(),
    ];
    let code = commit::run(&args);
    let _ = fs::remove_file(&message_file);
    if code != 0 {
        // Откатить дописанные события: событие без коммита — не история.
        rollback(repo, &file, &previous, existed);
    }
    Ok(code)
}

/// Откат записи журнала (работа w-journal-transaction): файл возвращается в
/// прежнее состояние — вплоть до отсутствия, — и его staged-копия
/// вычищается из индекса git, чтобы сбой не оставлял следа в репозитории.
fn rollback(repo: &git::Repo, file: &Path, previous: &str, existed: bool) {
    if existed {
        let _ = fs::write(file, previous);
    } else {
        let _ = fs::remove_file(file);
    }
    if let Ok(relative) = file.strip_prefix(&repo.root) {
        git::succeeds(
            &repo.root,
            &["reset", "-q", "--", &relative.to_string_lossy()],
        );
    }
}

/// События журнала рабочего дерева: одна запись и прежний каталог вместе.
fn journal_events(repo: &git::Repo) -> Vec<Event> {
    let mut events = Vec::new();
    let file = repo.root.join(repo.config.journal_file());
    let dir = repo.root.join(repo.config.journal_dir());
    if let Ok((file_events, _)) = dacc_journal::read_file(&file) {
        events.extend(file_events);
    }
    if dir.is_dir() {
        if let Ok((dir_events, _)) = dacc_journal::read_dir(&dir) {
            events.extend(dir_events);
        }
    }
    events
}

/// Дописывает готовые события и коммитит их под локом записи журнала.
fn record_and_commit(repo: &git::Repo, events: Vec<Event>, message: &str) -> Result<u8, Refusal> {
    let message = message.to_owned();
    record_and_commit_under_lock(repo, move || Ok((events, message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Радиус читается из текста работы: `radius: BlastRadius::Xxx`.
    #[test]
    fn record_parses_radius() {
        let crate_radius =
            Record::parse("dacc_work::work!(\"w-x\",\n    radius: BlastRadius::Crate,\n);");
        assert_eq!(crate_radius.radius.as_deref(), Some("crate"));
        let local = Record::parse("dacc_work::work!(\"w-x\",\n    radius: BlastRadius::Local,\n);");
        assert_eq!(local.radius.as_deref(), Some("local"));
    }

    /// Происхождение Divergence читается из текста работы: по нему приземление
    /// требует RedBefore — вида доказательства «тест падал до починки».
    #[test]
    fn record_parses_divergence() {
        let defect = Record::parse(
            "dacc_work::work!(\"w-x\",\n    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_x },\n);",
        );
        assert!(defect.divergence);
        let plain = Record::parse("dacc_work::work!(\"w-x\",\n    radius: BlastRadius::Local,\n);");
        assert!(!plain.divergence);
    }

    /// Индексы стадий и видов ложатся на счётчики `metrics`.
    #[test]
    fn stage_and_kind_indexes() {
        assert_eq!(stage_index(Stage::Planned), 0);
        assert_eq!(stage_index(Stage::Started), 1);
        assert_eq!(stage_index(Stage::Landed), 2);
        assert_eq!(stage_index(Stage::LandedFromHistory), 3);
        assert_eq!(stage_index(Stage::Abandoned), 4);
        assert_eq!(kind_index(&Kind::Started), 0);
        assert_eq!(kind_index(&Kind::Closed), 4);
    }

    /// Метрика считает коммиты с основанием, а не строки трейлеров: коммит с
    /// двумя трейлерами несёт одно основание (работа w-access-answers).
    #[test]
    fn metrics_counts_commits_not_trailers() {
        let log = concat!(
            "\u{1e}abc\n[FEAT](cli): x\n\nDacc-Work: w-001\nDacc-Slice: s-001\n",
            "\u{1e}def\n[FEAT](cli): y\n\nDacc-Work: w-002\n",
            "\u{1e}ghi\n[FEAT](cli): z\n"
        );
        assert_eq!(basis_commits(log), 2);
    }

    /// Тексты стадий — полный перечень автомата, и stage_text берёт их отсюда.
    #[test]
    fn stage_texts_cover_every_stage() {
        for stage in [
            Stage::Planned,
            Stage::Started,
            Stage::Landed,
            Stage::LandedFromHistory,
            Stage::Abandoned,
        ] {
            assert_eq!(stage_text(stage), STAGE_TEXTS[stage_index(stage)]);
        }
    }

    /// Запись upgrade! читает from/to/subject/how из текста файла.
    #[test]
    fn upgrade_record_parses_fields() {
        let record = UpgradeRecord::parse(
            "dacc_work::upgrade!(\"u-x\",\n    from: NonEmptyStr::new(\"0.3.0\"),\n    to: NonEmptyStr::new(\"0.4.0\"),\n    subject: NonEmptyStr::new(\"s\"),\n    how: NonEmptyStr::new(\"h\"),\n);",
        )
        .unwrap();
        assert_eq!(record.from, "0.3.0");
        assert_eq!(record.to, "0.4.0");
        assert_eq!(record.subject, "s");
        assert_eq!(record.how, "h");
    }
}
