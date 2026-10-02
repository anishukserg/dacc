//! Генератор `work new` (решение 5): пишет файл единицы работы по аргументам,
//! а по первым десяти единицам записывает замер трения — время заведения и
//! число циклов сборки.

use crate::code::{self, Code};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Аргументы `work new`, разобранные в поля схемы единицы работы.
struct WorkSpec {
    id: String,
    slice: String,
    title: String,
    outcome: String,
    /// Имя подсистемы, как в таксономии: `Cli`, `Work`, …
    taxon: String,
    /// Имя радиуса: `Local`, `Crate`, …
    radius: String,
    origin: Origin,
    /// Число циклов сборки до первой успешной (по умолчанию один).
    cycles: u32,
}

/// Происхождение единицы работы — один вариант `WorkOrigin`.
enum Origin {
    Decision(String),
    Specification(String),
    Divergence {
        rfc: String,
        limitation: String,
    },
    Inquiry {
        question: String,
        produces: String,
        timebox: u16,
    },
    Retirement(String),
    Toil(String),
}

/// Дополнительные аргументы происхождения, общие для всех вариантов.
struct OriginArgs {
    adr: Option<String>,
    rfc: Option<String>,
    limitation: Option<String>,
    question: Option<String>,
    produces: Option<String>,
    timebox: Option<String>,
    superseded: Option<String>,
    justification: Option<String>,
}

/// `cargo dacc work new <w-slug> --slice <s-slug> --title … --outcome …
/// --taxon … --radius … --origin …`.
pub fn work_new(words: &[String]) -> Result<u8, crate::work::Refusal> {
    let spec = WorkSpec::parse(words)?;
    let context = crate::work::Context::open()?;
    crate::work::ensure_no_drift(&context.repo)?;
    let path = context
        .repo
        .root
        .join(context.repo.config.work_dir())
        .join(format!("{}.rs", spec.id));
    if path.exists() {
        return Err(refused(
            code::USAGE,
            format!("{} already exists: {}", spec.id, path.display()),
        ));
    }
    let text = spec.render();
    fs::write(&path, text).map_err(|error| {
        usage(
            code::EVENT_NOT_WRITTEN,
            format!("{} not written: {error}", path.display()),
        )
    })?;
    // Замер пишется после файла работы; при отказе замера файл работы
    // откатывается — единица работы не остаётся в дереве наполовину.
    if let Err(problem) = record_measurement(
        &context.repo.root,
        &context.repo.config.doc,
        &spec.id,
        spec.cycles,
    ) {
        let _ = fs::remove_file(&path);
        return Err(problem);
    }
    println!("WORK NEW {} {}", spec.id, path.display());
    Ok(0)
}

/// Замер трения (решение 5): одна строка на единицу — время, slug, циклы сборки.
/// Пишется в `<doc>/work-new.tsv`, где `<doc>` — корень реестра из настройки:
/// замер живёт рядом с единицами работы, а не в захардкоженном каталоге.
fn record_measurement(
    root: &Path,
    doc: &str,
    id: &str,
    cycles: u32,
) -> Result<(), crate::work::Refusal> {
    let path = root.join(doc).join("work-new.tsv");
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| {
            usage(
                code::EVENT_NOT_WRITTEN,
                format!("{} not created: {error}", dir.display()),
            )
        })?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| {
            usage(
                code::EVENT_NOT_WRITTEN,
                format!("{} not opened: {error}", path.display()),
            )
        })?;
    writeln!(file, "{}\t{id}\t{cycles}", dacc_journal::time::now()).map_err(|error| {
        usage(
            code::EVENT_NOT_WRITTEN,
            format!("{} not written: {error}", path.display()),
        )
    })?;
    Ok(())
}

fn refused(code: Code, reason: impl Into<String>) -> crate::work::Refusal {
    crate::work::refused(code, reason)
}

fn usage(code: Code, reason: impl Into<String>) -> crate::work::Refusal {
    crate::work::usage(code, reason)
}

impl WorkSpec {
    fn parse(words: &[String]) -> Result<WorkSpec, crate::work::Refusal> {
        let mut id = None;
        let mut slice = None;
        let mut title = None;
        let mut outcome = None;
        let mut taxon = None;
        let mut radius = None;
        let mut origin_type = None;
        let mut adr = None;
        let mut rfc = None;
        let mut limitation = None;
        let mut question = None;
        let mut produces = None;
        let mut timebox = None;
        let mut superseded = None;
        let mut justification = None;
        let mut cycles = 1u32;

        let mut iter = words.iter();
        while let Some(word) = iter.next() {
            let mut value = |name: &str| {
                iter.next()
                    .ok_or_else(|| usage(code::FLAG_NEEDS_VALUE, format!("{name} needs a value")))
            };
            match word.as_str() {
                "--slice" => slice = Some(value("--slice")?.clone()),
                "--title" => title = Some(value("--title")?.clone()),
                "--outcome" => outcome = Some(value("--outcome")?.clone()),
                "--taxon" => taxon = Some(value("--taxon")?.clone()),
                "--radius" => radius = Some(value("--radius")?.clone()),
                "--origin" => origin_type = Some(value("--origin")?.clone()),
                "--adr" => adr = Some(value("--adr")?.clone()),
                "--rfc" => rfc = Some(value("--rfc")?.clone()),
                "--limitation" => limitation = Some(value("--limitation")?.clone()),
                "--question" => question = Some(value("--question")?.clone()),
                "--produces" => produces = Some(value("--produces")?.clone()),
                "--timebox" => timebox = Some(value("--timebox")?.clone()),
                "--superseded" => superseded = Some(value("--superseded")?.clone()),
                "--justification" => justification = Some(value("--justification")?.clone()),
                "--cycles" => {
                    let raw = value("--cycles")?;
                    cycles = raw
                        .parse::<u32>()
                        .map_err(|_| usage(code::USAGE, "--cycles needs a number"))?;
                }
                flag if flag.starts_with("--") => {
                    return Err(usage(
                        code::UNKNOWN_ARGUMENT,
                        format!("unknown argument {flag}"),
                    ))
                }
                _ if id.is_none() => id = Some(word.clone()),
                _ => {
                    return Err(usage(
                        code::UNKNOWN_ARGUMENT,
                        format!("extra argument {word}"),
                    ))
                }
            }
        }

        let id =
            id.ok_or_else(|| usage(code::USAGE, "work new needs a work id like w-fix-gitignore"))?;
        if !id.starts_with("w-") {
            return Err(usage(
                code::NOT_WORK_ID,
                format!("{id} is not a work id of the form w-slug"),
            ));
        }
        let slice = slice.ok_or_else(|| usage(code::USAGE, "--slice <s-slug> is required"))?;
        let title = title.ok_or_else(|| usage(code::USAGE, "--title is required"))?;
        let outcome = outcome.ok_or_else(|| usage(code::USAGE, "--outcome is required"))?;
        let taxon =
            normalize_taxon(&taxon.ok_or_else(|| usage(code::USAGE, "--taxon is required"))?)?;
        let radius =
            normalize_radius(&radius.ok_or_else(|| usage(code::USAGE, "--radius is required"))?)?;
        let origin = parse_origin(
            origin_type.as_deref(),
            &OriginArgs {
                adr,
                rfc,
                limitation,
                question,
                produces,
                timebox,
                superseded,
                justification,
            },
        )?;

        Ok(WorkSpec {
            id,
            slice,
            title,
            outcome,
            taxon,
            radius,
            origin,
            cycles,
        })
    }

    fn render(&self) -> String {
        let slice = ident(&self.slice);
        let origin = self.render_origin();
        format!(
            "use crate::taxonomy::Subsystem;\n{imports}dacc_work::work!({id:?},\n    title: NonEmptyStr::new({title}),\n    slice: crate::slice::{slice},\n    origin: {origin},\n    taxon: taxon!(Subsystem, {taxon}),\n    radius: BlastRadius::{radius},\n    outcome: NonEmptyStr::new({outcome}),\n);\n",
            imports = self.render_imports(),
            id = self.id,
            title = rust_string(&self.title),
            slice = slice,
            origin = origin,
            taxon = self.taxon,
            radius = self.radius,
            outcome = rust_string(&self.outcome),
        )
    }

    fn render_imports(&self) -> String {
        match self.origin {
            Origin::Inquiry { .. } => {
                "use dacc_core::{taxon, BlastRadius, NonEmptyStr};\nuse dacc_work::{InquiryOutcome, WorkOrigin};\nuse std::num::NonZeroU16;\n\n".to_owned()
            }
            _ => "use dacc_core::{taxon, BlastRadius, NonEmptyStr};\nuse dacc_work::WorkOrigin;\n\n".to_owned(),
        }
    }

    fn render_origin(&self) -> String {
        match &self.origin {
            Origin::Decision(adr) => {
                format!("WorkOrigin::Decision(crate::adr::{})", ident(adr))
            }
            Origin::Specification(rfc) => {
                format!("WorkOrigin::Specification(crate::rfc::{})", ident(rfc))
            }
            Origin::Divergence { rfc, limitation } => format!(
                "WorkOrigin::Divergence {{ specification: crate::rfc::{}, limitation: crate::limitation::{} }}",
                ident(rfc),
                ident(limitation)
            ),
            Origin::Inquiry {
                question,
                produces,
                timebox,
            } => format!(
                "WorkOrigin::Inquiry {{ question: NonEmptyStr::new({}), produces: InquiryOutcome::{}, timebox_days: NonZeroU16::new({timebox}).unwrap() }}",
                rust_string(question),
                produces
            ),
            Origin::Retirement(superseded) => format!(
                "WorkOrigin::Retirement(crate::superseded::{})",
                ident(superseded)
            ),
            Origin::Toil(justification) => format!(
                "WorkOrigin::Toil {{ justification: NonEmptyStr::new({}) }}",
                rust_string(justification)
            ),
        }
    }
}

fn parse_origin(
    origin_type: Option<&str>,
    args: &OriginArgs,
) -> Result<Origin, crate::work::Refusal> {
    match origin_type {
        Some("decision") => Ok(Origin::Decision(args.adr.clone().ok_or_else(|| {
            usage(code::USAGE, "--origin decision needs --adr <adr-slug>")
        })?)),
        Some("specification") => Ok(Origin::Specification(args.rfc.clone().ok_or_else(|| {
            usage(code::USAGE, "--origin specification needs --rfc <rfc-slug>")
        })?)),
        Some("divergence") => Ok(Origin::Divergence {
            rfc: args
                .rfc
                .clone()
                .ok_or_else(|| usage(code::USAGE, "--origin divergence needs --rfc <rfc-slug>"))?,
            limitation: args
                .limitation
                .clone()
                .ok_or_else(|| usage(code::USAGE, "--origin divergence needs --limitation <l-slug>"))?,
        }),
        Some("inquiry") => {
            let produces = args
                .produces
                .clone()
                .ok_or_else(|| usage(code::USAGE, "--origin inquiry needs --produces"))?;
            if !matches!(produces.as_str(), "Adr" | "Rfc" | "Measurement") {
                return Err(usage(code::USAGE, "--produces is Adr, Rfc or Measurement"));
            }
            let timebox = args
                .timebox
                .as_deref()
                .and_then(|value| value.parse::<u16>().ok())
                .ok_or_else(|| usage(code::USAGE, "--origin inquiry needs --timebox <days>"))?;
            Ok(Origin::Inquiry {
                question: args
                    .question
                    .clone()
                    .ok_or_else(|| usage(code::USAGE, "--origin inquiry needs --question"))?,
                produces,
                timebox,
            })
        }
        Some("retirement") => Ok(Origin::Retirement(
            args.superseded
                .clone()
                .ok_or_else(|| usage(code::USAGE, "--origin retirement needs --superseded <adr-slug>"))?,
        )),
        Some("toil") => Ok(Origin::Toil(args.justification.clone().ok_or_else(|| {
            usage(code::USAGE, "--origin toil needs --justification")
        })?)),
        Some(other) => Err(usage(
            code::USAGE,
            format!(
                "--origin is decision, specification, divergence, inquiry, retirement or toil, not {other}"
            ),
        )),
        None => Err(usage(code::USAGE, "--origin is required")),
    }
}

fn normalize_taxon(value: &str) -> Result<String, crate::work::Refusal> {
    match value {
        "knowledge" => Ok("Knowledge".to_owned()),
        "work" => Ok("Work".to_owned()),
        "access" => Ok("Access".to_owned()),
        "scan" => Ok("Scan".to_owned()),
        "cli" => Ok("Cli".to_owned()),
        "methodology" => Ok("Methodology".to_owned()),
        other => Err(usage(
            code::USAGE,
            format!(
                "--taxon is a subsystem: knowledge, work, access, scan, cli or methodology, not {other}"
            ),
        )),
    }
}

fn normalize_radius(value: &str) -> Result<String, crate::work::Refusal> {
    match value {
        "local" => Ok("Local".to_owned()),
        "crate" => Ok("Crate".to_owned()),
        "contract" => Ok("Contract".to_owned()),
        "persistent" => Ok("Persistent".to_owned()),
        "irreversible" => Ok("Irreversible".to_owned()),
        other => Err(usage(
            code::USAGE,
            format!("--radius is local, crate, contract, persistent or irreversible, not {other}"),
        )),
    }
}

/// Строковый литерал Rust в двойных кавычках, с экранированием.
fn rust_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Rust-идентификатор из slug: дефисы заменяются подчёркиваниями.
fn ident(slug: &str) -> String {
    slug.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(origin: Origin) -> WorkSpec {
        WorkSpec {
            id: "w-fix-gitignore".to_owned(),
            slice: "s-minimal-work-layer".to_owned(),
            title: "Поправить gitignore".to_owned(),
            outcome: "Файл записан".to_owned(),
            taxon: "Cli".to_owned(),
            radius: "Local".to_owned(),
            origin,
            cycles: 1,
        }
    }

    #[test]
    fn renders_a_decision_work_file() {
        let text = spec(Origin::Decision("adr-2026-005".to_owned())).render();
        assert!(
            text.contains("dacc_work::work!(\"w-fix-gitignore\","),
            "{text}"
        );
        assert!(
            text.contains("slice: crate::slice::s_minimal_work_layer,"),
            "{text}"
        );
        assert!(
            text.contains("origin: WorkOrigin::Decision(crate::adr::adr_2026_005),"),
            "{text}"
        );
        assert!(text.contains("taxon: taxon!(Subsystem, Cli),"), "{text}");
        assert!(text.contains("radius: BlastRadius::Local,"), "{text}");
        assert!(!text.contains("InquiryOutcome"), "{text}");
    }

    #[test]
    fn renders_an_inquiry_work_file() {
        let text = spec(Origin::Inquiry {
            question: "Что?".to_owned(),
            produces: "Adr".to_owned(),
            timebox: 5,
        })
        .render();
        assert!(
            text.contains("use dacc_work::{InquiryOutcome, WorkOrigin};"),
            "{text}"
        );
        assert!(text.contains("use std::num::NonZeroU16;"), "{text}");
        assert!(text.contains("produces: InquiryOutcome::Adr,"), "{text}");
        assert!(
            text.contains("timebox_days: NonZeroU16::new(5).unwrap()"),
            "{text}"
        );
    }

    #[test]
    fn escapes_quotes_backslashes_and_newlines() {
        assert_eq!(rust_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(rust_string("a\\b"), "\"a\\\\b\"");
        assert_eq!(rust_string("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn parses_a_decision_invocation() {
        let words = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let parsed = WorkSpec::parse(&words(&[
            "w-fix",
            "--slice",
            "s-minimal-work-layer",
            "--title",
            "Заголовок",
            "--outcome",
            "Исход",
            "--taxon",
            "cli",
            "--radius",
            "local",
            "--origin",
            "decision",
            "--adr",
            "adr-2026-005",
        ]))
        .unwrap();
        assert_eq!(parsed.taxon, "Cli");
        assert_eq!(parsed.radius, "Local");
        assert!(matches!(parsed.origin, Origin::Decision(_)));
    }
}
