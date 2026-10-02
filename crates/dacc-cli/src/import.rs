//! Приём журнала из истории по подтверждению человека (решение 15, работа
//! w-import-attestation).
//!
//! ```text
//! cargo dacc journal import --work <w-slug> [--land <w-slug>[=<commit>]]…
//!     [--close-finished-slices]
//! ```
//!
//! Импорт не угадывает: каждое приземление из истории подтверждает человек
//! аргументом `--land`. `--land <work>` берёт последний коммит с трейлером
//! работы, `--land <work>=<commit>` называет коммит явно — так восстанавливается
//! история до трейлеров. Без подтверждений команда отказывает и называет
//! кандидатов. Основание импорта не подтверждается: оно ещё в работе.

use crate::code;
use crate::config;
use crate::git;
use crate::proof;
use crate::work::{
    finish, message, record_and_commit, refused, short, stage_text, usage, words, work_number,
    Context, Refusal,
};
use dacc_journal::{time, Event, Evidence, Kind, Stage, Subject};
use std::ffi::OsString;

const IMPORT_USAGE: &str =
    "journal import --work <w-slug> [--land <w-slug>[=<commit>]]… [--close-finished-slices]";

/// `cargo dacc journal import …`.
pub fn run_import(args: &[OsString]) -> u8 {
    finish(words(args).and_then(|(words, trailers)| {
        let mut basis: Option<String> = None;
        let mut attestations: Vec<String> = Vec::new();
        let mut close_slices = false;
        let mut rest = words.into_iter();
        while let Some(word) = rest.next() {
            match word.as_str() {
                "--work" => {
                    if basis.is_some() {
                        return Err(usage(code::USAGE, "--work is stated once"));
                    }
                    basis = Some(
                        rest.next()
                            .ok_or_else(|| usage(code::USAGE, "--work needs a work id"))?,
                    );
                }
                "--land" => {
                    attestations.push(rest.next().ok_or_else(|| {
                        usage(code::USAGE, "--land needs a work id or work=commit")
                    })?)
                }
                "--close-finished-slices" => close_slices = true,
                _ => return Err(usage(code::USAGE, IMPORT_USAGE)),
            }
        }
        let Some(basis) = basis else {
            return Err(usage(code::USAGE, IMPORT_USAGE));
        };
        import(&basis, close_slices, &attestations, &trailers)
    }))
}

/// Приземления из истории по подтверждению человека (решение 15, работа
/// w-import-attestation): `--land <work>` приземляет работу по последнему
/// коммиту с её трейлером, `--land <work>=<commit>` — по названному коммиту,
/// включая историю до трейлеров. Импорт не угадывает: без подтверждений он
/// отказывает и называет кандидатов. Основание импорта не подтверждается: оно
/// ещё в работе. С `close_slices` закрываются срезы, все работы которых после
/// импорта завершены.
fn import(
    basis: &str,
    close_slices: bool,
    attestations: &[String],
    trailers: &[String],
) -> Result<u8, Refusal> {
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

    // Подтверждения: работа и коммит названы человеком, а не выведены импортом.
    // `--land <work>=<commit>` покрывает историю до трейлеров.
    let mut attested = std::collections::BTreeMap::new();
    for spec in attestations {
        let (id, revision) = match spec.split_once('=') {
            Some((id, revision)) => (id, Some(revision)),
            None => (spec.as_str(), None),
        };
        let number = work_number(id)?;
        if number == basis_number {
            return Err(refused(
                code::IMPORT_BASIS,
                format!("the basis of the import cannot attest its own landing: {id}"),
            ));
        }
        if !works.iter().any(|(known, _)| *known == number) {
            return Err(refused(
                code::WORK_NOT_IN_PLAN,
                format!("{id} is not in the plan"),
            ));
        }
        let stage = context.journal.stage(&number);
        if stage != Stage::Planned {
            return Err(refused(
                code::WORK_STAGE,
                format!("work {id} is already {}", stage_text(stage)),
            ));
        }
        let commit = match revision {
            Some(revision) => {
                let object = format!("{revision}^{{commit}}");
                let commit =
                    git::read(root, &["rev-parse", "--verify", "--quiet", &object]).ok_or_else(
                        || {
                            refused(
                                code::REVISION_NOT_COMMIT,
                                format!("revision {revision} is not a commit"),
                            )
                        },
                    )?;
                if !git::succeeds(root, &["merge-base", "--is-ancestor", &commit, "HEAD"]) {
                    return Err(refused(
                        code::COMMIT_NOT_ANCESTOR,
                        format!("commit {} is not in the history of HEAD", short(&commit)),
                    ));
                }
                commit
            }
            None => latest.get(&number).cloned().ok_or_else(|| {
                refused(
                    code::IMPORT_NO_COMMIT,
                    format!(
                        "work {id} has no commit carrying its trailer: attest the commit with --land {id}=<commit>"
                    ),
                )
            })?,
        };
        attested.insert(number, commit);
    }

    if attested.is_empty() {
        // Импорт не угадывает: кандидаты называются, но не приземляются.
        let candidates: Vec<String> = works
            .iter()
            .filter(|(number, _)| {
                *number != basis_number
                    && context.journal.stage(number) == Stage::Planned
                    && latest.contains_key(number)
            })
            .map(|(number, _)| number.clone())
            .collect();
        return Err(if candidates.is_empty() {
            refused(
                code::NOTHING_TO_IMPORT,
                "nothing to import: planned works have no commits carrying their trailer",
            )
        } else {
            refused(
                code::IMPORT_UNATTESTED,
                format!(
                    "import does not guess: {} planned works have commits carrying their trailer ({}) — attest each landing with --land <work> or --land <work>=<commit>",
                    candidates.len(),
                    candidates.join(", ")
                ),
            )
        });
    }

    for (number, commit) in &attested {
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
