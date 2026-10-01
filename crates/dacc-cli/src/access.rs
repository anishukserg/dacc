//! Слой доступа (решение 21, RFC-0003): карта реестра, место в коде и
//! директивы агентов. Ответы идут из дерева реестра и свёртки журнала, а не
//! из обхода каталогов и текстового поиска: навигация не требует find, grep
//! и ls.

use crate::format::{self, Format};
use crate::work::{quoted_after, stage_text, usage, Context, Record, Refusal};
use crate::{code, work};
use std::ffi::OsString;
use std::fs;

/// `cargo dacc map [--format json]` — карта реестра одной командой.
pub fn run_map(args: &[OsString]) -> u8 {
    work::finish(map(args))
}

/// Направление карты.
struct Thrust {
    id: String,
    title: String,
    outcome: String,
}

/// Срез карты.
struct Slice {
    id: String,
    title: String,
    outcome: String,
    thrust: String,
    closed: bool,
}

/// Единица работы карты.
struct Work {
    id: String,
    title: String,
    slice: Option<String>,
    taxon: Option<String>,
    radius: Option<String>,
    state: String,
}

fn map(args: &[OsString]) -> Result<u8, Refusal> {
    let format = scan_format(args)?;
    let context = Context::open()?;
    let root = &context.repo.root;

    let mut thrusts = Vec::new();
    for (id, text) in records(&root.join(format!("{}/thrust", context.repo.config.doc))) {
        thrusts.push(Thrust {
            outcome: quoted_after(&text, "outcome: NonEmptyStr::new(").unwrap_or_default(),
            title: quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default(),
            id,
        });
    }

    let mut slices = Vec::new();
    for (id, text) in records(&root.join(context.repo.config.slice_dir())) {
        let record = Record::parse(&text);
        slices.push(Slice {
            closed: context.journal.closed_slices.contains_key(&id),
            thrust: ref_after(&text, "thrust: crate::thrust::").unwrap_or_default(),
            outcome: quoted_after(&text, "outcome: NonEmptyStr::new(").unwrap_or_default(),
            title: record.title,
            id,
        });
    }

    let mut works = Vec::new();
    for (id, record) in context.works()? {
        works.push(Work {
            state: stage_text(context.journal.stage(&id)).to_owned(),
            slice: record.slice.clone(),
            taxon: record.area.clone(),
            radius: record.radius.clone(),
            title: record.title,
            id,
        });
    }

    let decisions = count_files(&root.join(format!("{}/adr", context.repo.config.doc)));
    let specifications = count_files(&root.join(format!("{}/rfc", context.repo.config.doc)));

    match format {
        Format::Json => {
            println!(
                "{}",
                render_json(&thrusts, &slices, &works, decisions, specifications)
            );
        }
        Format::Text => {
            print!(
                "{}",
                render_text(&thrusts, &slices, &works, decisions, specifications)
            );
        }
    }
    Ok(0)
}

/// `--format` из аргументов; лишнее — отказ с перечнем принятого.
fn scan_format(args: &[OsString]) -> Result<Format, Refusal> {
    let mut format = Format::Text;
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        let flag = flag.to_string_lossy();
        if flag != "--format" {
            return Err(usage(
                code::USAGE,
                "map [--format json] — unknown argument {flag}",
            ));
        }
        let value = words
            .next()
            .ok_or_else(|| usage(code::FORMAT_CHOICE, "--format needs `json` or `text`"))?;
        format = format::parse(&value.to_string_lossy())
            .map_err(|problem| usage(code::FORMAT_CHOICE, problem))?;
    }
    Ok(format)
}

/// Файлы записей каталога: имя файла без расширения — slug, текст — запись.
fn records(dir: &std::path::Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let id = path.file_stem()?.to_string_lossy().into_owned();
            let text = fs::read_to_string(&path).ok()?;
            Some((id, text))
        })
        .collect()
}

/// Имя файла из числа файлов каталога.
fn count_files(dir: &std::path::Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .count()
}

/// Идентификатор после маркера ссылки: `thrust: crate::thrust::t-001`.
fn ref_after(text: &str, marker: &str) -> Option<String> {
    let at = text.find(marker)?;
    let rest = &text[at + marker.len()..];
    let end = rest.find([',', ')', '\n']).unwrap_or(rest.len());
    let slug = rest[..end].trim();
    (!slug.is_empty()).then(|| slug.to_owned())
}

/// Карта текстом: дерево направлений, срезов и работ и счётчики.
fn render_text(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut out = String::new();
    for thrust in thrusts {
        out.push_str(&format!("{} — {}\n", thrust.id, thrust.title));
        out.push_str(&format!("    {}\n", thrust.outcome));
        for slice in slices.iter().filter(|slice| slice.thrust == thrust.id) {
            let state = if slice.closed { "closed" } else { "open" };
            out.push_str(&format!("  {} [{}] — {}\n", slice.id, state, slice.title));
            out.push_str(&format!("      {}\n", slice.outcome));
            for work in works
                .iter()
                .filter(|work| work.slice.as_deref() == Some(slice.id.as_str()))
            {
                out.push_str(&format!(
                    "    {} [{}] — {}\n",
                    work.id, work.state, work.title
                ));
            }
        }
    }
    let open = slices.iter().filter(|slice| !slice.closed).count();
    out.push_str(&format!(
        "thrusts: {}, slices: {} (open {}, closed {}), works: {}\n",
        thrusts.len(),
        slices.len(),
        open,
        slices.len() - open,
        works.len()
    ));
    let mut stages: Vec<String> = Vec::new();
    for state in ["planned", "started", "landed", "abandoned"] {
        let count = works.iter().filter(|work| work.state == state).count();
        if count > 0 {
            stages.push(format!("{state} {count}"));
        }
    }
    out.push_str(&format!("works by state: {}\n", stages.join(", ")));
    out.push_str(&format!(
        "decisions: {decisions}, specifications: {specifications}\n"
    ));
    out
}

/// Карта машинным форматом: те же данные полями.
fn render_json(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut out = String::from("{\"schema\": \"dacc-map\", \"thrusts\": [");
    for (i, thrust) in thrusts.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"outcome\": {}}}",
            json_string(&thrust.id),
            json_string(&thrust.title),
            json_string(&thrust.outcome)
        ));
    }
    out.push_str("], \"slices\": [");
    for (i, slice) in slices.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"thrust\": {}, \"closed\": {}, \"outcome\": {}}}",
            json_string(&slice.id),
            json_string(&slice.title),
            json_string(&slice.thrust),
            slice.closed,
            json_string(&slice.outcome)
        ));
    }
    out.push_str("], \"works\": [");
    for (i, work) in works.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"slice\": {}, \"taxon\": {}, \"radius\": {}, \"state\": {}}}",
            json_string(&work.id),
            json_string(&work.title),
            work.slice
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.taxon
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.radius
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            json_string(&work.state)
        ));
    }
    out.push_str(&format!(
        "], \"decisions\": {decisions}, \"specifications\": {specifications}}}\n"
    ));
    out
}

/// Строка JSON с экранированием управляющих символов.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
