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
use crate::{commit, config, gate, git, hooks, layout, proof};
use dacc_journal::{
    fold, time, Event, Evidence, GateVerdict, Journal, Kind, Proof, Stage, Subject,
};
use std::ffi::OsString;
use std::fs;
use std::path::Path;

const WORK_USAGE: &str = "work start <w-slug> | new <w-slug> --slice … --origin … | land <w-slug> [--commit <revision>] [--red-before <subject>] [--mutation-proof <subject>] [--anti-vacuum <subject>] | drop <w-slug> --reason <reason> | state [<w-slug>]";

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
    let total = git::read(root, &["rev-list", "--count", "HEAD"])
        .and_then(|out| out.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let with_basis = git::read(root, &["log", "--format=%s%n%b"])
        .map(|log| {
            log.lines()
                .filter(|line| line.starts_with("Dacc-Work:") || line.starts_with("Dacc-Slice:"))
                .count()
        })
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
            from: quoted_after(text, "from: NonEmptyStr::new(")?,
            to: quoted_after(text, "to: NonEmptyStr::new(")?,
            subject: quoted_after(text, "subject: NonEmptyStr::new(")?,
            how: quoted_after(text, "how: NonEmptyStr::new(")?,
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
        Format::Text | Format::Xml => {
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
        let anchors = flat(field_value(&fields, "enforced_by").unwrap_or_default());
        let list = anchors.trim_start_matches('&');
        let inner = list
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
            .unwrap_or(list);
        if inner.is_empty() {
            return Err(refused(
                code::CONTRACT_UNENFORCED,
                format!("invariant {record_id} of {spec} has no live anchor"),
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

/// Мини-разбор записей реестра значениями полей (работа w-record-parse):
/// комментарии снимаются по лексике строк, значение читается сбалансированной
/// группой до запятой верхнего уровня. Форма записи не даёт спрятать значение
/// в комментарий, перенос строки или мультилайн.
pub(crate) fn record_fields(text: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = strip_comments(text).chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut depth = 0i32;
    while i < chars.len() {
        match chars[i] {
            '"' => i = skip_string(&chars, i),
            '(' | '[' | '{' => {
                depth += 1;
                i += 1;
            }
            ')' | ']' | '}' => {
                depth -= 1;
                i += 1;
            }
            c if depth == 1 && (c.is_ascii_alphabetic() || c == '_') => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if chars.get(j) != Some(&':') {
                    continue;
                }
                i = j + 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                let vstart = i;
                let mut level = 0i32;
                while i < chars.len() {
                    match chars[i] {
                        '"' => i = skip_string(&chars, i),
                        '(' | '[' | '{' => {
                            level += 1;
                            i += 1;
                        }
                        ')' | ']' | '}' if level == 0 => break,
                        ')' | ']' | '}' => {
                            level -= 1;
                            i += 1;
                        }
                        ',' if level == 0 => break,
                        _ => i += 1,
                    }
                }
                let value: String = chars[vstart..i].iter().collect();
                out.push((name, value.trim().to_owned()));
            }
            _ => i += 1,
        }
    }
    out
}

/// Текст записи без комментариев: снятие идёт по лексике, строковые литералы
/// не трогаются — значение не спрятать в комментарий.
fn strip_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            let end = skip_string(&chars, i);
            out.extend(&chars[i..end]);
            i = end;
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Индекс за закрывающей кавычкой строкового литерала.
fn skip_string(chars: &[char], start: usize) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
        } else if chars[i] == '"' {
            return i + 1;
        } else {
            i += 1;
        }
    }
    i
}

/// Значение поля записи.
fn field_value<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// Текст без пробелов: сравнение путей и идентификаторов без формы.
fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Первый строковый литерал значения — заголовок записи.
fn string_literal(value: &str) -> Option<String> {
    let start = value.find('"')?;
    let mut out = String::new();
    let mut chars = value[start + 1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push(chars.next()?),
            '"' => return Some(out),
            c => out.push(c),
        }
    }
    None
}

/// Последний сегмент пути `a::b::c` — идентификатор записи.
fn last_ident(value: &str) -> Option<String> {
    let ident = value
        .rsplit("::")
        .next()?
        .trim()
        .trim_end_matches(')')
        .trim();
    (!ident.is_empty()).then(|| ident.to_owned())
}

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
            slice: field_value(&fields, "slice")
                .and_then(last_ident)
                .map(|slug| slug.replace('_', "-")),
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

/// Строка в кавычках после `marker`; между маркером и кавычкой допустимы
/// пробелы и переводы строк.
pub(crate) fn quoted_after(text: &str, marker: &str) -> Option<String> {
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
pub(crate) fn work_number(id: &str) -> Result<String, Refusal> {
    match Subject::parse(id) {
        Some(Subject::Work(number)) => Ok(number),
        _ => Err(usage(
            code::NOT_WORK_ID,
            format!("{id} is not a work id of the form w-fix-gitignore"),
        )),
    }
}

pub(crate) fn stage_text(stage: Stage) -> &'static str {
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
fn message(subject: &str, body: &str, basis: &str, trailers: &[String]) -> String {
    let mut text = format!("{subject}\n\n{body}\n\n{basis}\n");
    for trailer in trailers {
        text.push_str(trailer);
        text.push('\n');
    }
    text
}

/// Дописывает события в одну запись журнала `journal.toml` и коммитит ровно её
/// (решение 43). Если коммит не создан, дописанное откатывается: событие без
/// коммита — не история.
fn record_and_commit(repo: &git::Repo, events: Vec<Event>, message: &str) -> Result<u8, Refusal> {
    // Межпроцессный лок записи (работа w-journal-lock): параллельный процесс
    // получает отказ с кодом причины, а не теряет событие в гонке за журнал.
    let _lock = crate::commit::Lock::acquire(
        &repo.git_dir.join(layout::JOURNAL_LOCK),
        std::time::Duration::ZERO,
    )
    .map_err(|problem| {
        refused(
            code::LOCK_NOT_ACQUIRED,
            format!("journal write is locked: {problem}"),
        )
    })?;
    let file = repo.root.join(repo.config.journal_file());
    // Append-only: прежняя версия записи — префикс новой, поэтому читаем текущую
    // и дописываем события таблицами [[events]].
    let previous = fs::read_to_string(&file).unwrap_or_default();
    let mut text = previous.clone();
    for event in &events {
        text.push_str(&event.to_table());
    }
    if let Err(error) = fs::write(&file, &text) {
        return Err(usage(
            code::EVENT_NOT_WRITTEN,
            format!("{} not written: {error}", file.display()),
        ));
    }
    let message_file = repo.git_dir.join(layout::JOURNAL_MESSAGE);
    if let Err(error) = fs::write(&message_file, message) {
        let _ = fs::write(&file, previous);
        return Err(usage(
            code::COMMIT_MESSAGE_NOT_WRITTEN,
            format!("commit message not written: {error}"),
        ));
    }
    let args = vec![
        OsString::from("-F"),
        message_file.clone().into_os_string(),
        OsString::from("--"),
        file.strip_prefix(&repo.root)
            .unwrap_or(&file)
            .as_os_str()
            .to_owned(),
    ];
    let code = commit::run(&args);
    let _ = fs::remove_file(&message_file);
    if code != 0 {
        // Откатить дописанные события: событие без коммита — не история.
        let _ = fs::write(&file, previous);
    }
    Ok(code)
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
