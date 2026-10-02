//! `cargo dacc where --file <path> [--format json]` — что известно о месте
//! в коде: его разметка `#[doc_anchor]`, документы, ссылающиеся на неё, и
//! связанные работы.

use super::form::{json_string, xml_string};
use super::{records, scan_args, Work};
use crate::format::Format;
use crate::work::{stage_text, usage, Context, Record, Refusal};
use crate::{code, record};
use std::ffi::OsString;
use std::fs;

pub(super) fn run(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "where --file <path> [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() || scan.check {
        return Err(usage(code::USAGE, "where --file <path> [--format json]"));
    }
    let file = scan
        .file
        .ok_or_else(|| usage(code::USAGE, "where --file <path> is required"))?;
    let context = Context::open()?;
    let text =
        fs::read_to_string(context.repo.root.join(&file)).map_err(|_| refused_file(&file))?;

    // Разметка места — сама по себе ответ; без неё отдельная строка, а не
    // пустой успех (анти-вакуум).
    let anchors = anchors_in(&text);
    let mut documents = Vec::new();
    for (kind, dir) in super::KINDS {
        for (id, record) in records(
            &context
                .repo
                .root
                .join(format!("{}/{}", context.repo.config.doc, dir)),
        ) {
            let referenced = anchors.iter().any(|anchor| {
                record.contains(&format!("[{anchor}]")) || record.contains(&anchor_ident(anchor))
            });
            if referenced {
                documents.push(Document {
                    title: record::title(&record),
                    kind: (*kind).to_owned(),
                    id,
                });
            }
        }
    }
    let mut works = Vec::new();
    for (id, text) in records(&context.repo.root.join(context.repo.config.work_dir())) {
        let referenced = documents
            .iter()
            .any(|document| text.contains(&document.id.replace('-', "_")))
            || anchors
                .iter()
                .any(|anchor| text.contains(&anchor_ident(anchor)));
        if referenced {
            let parsed = Record::parse(&text);
            works.push(Work {
                state: stage_text(context.journal.stage(&id)).to_owned(),
                slice: parsed.slice,
                taxon: parsed.area,
                radius: parsed.radius,
                title: parsed.title,
                id,
            });
        }
    }

    match scan.format {
        Format::Json => println!("{}", where_json(&file, &anchors, &documents, &works)),
        Format::Text => print!("{}", where_text(&file, &anchors, &documents, &works)),
        Format::Xml => println!("{}", where_xml(&file, &anchors, &documents, &works)),
    }
    Ok(0)
}

fn refused_file(file: &str) -> Refusal {
    crate::work::refused(code::FILE_NOT_READ, format!("file {file} cannot be read"))
}

/// Документ, ссылающийся на якорь места.
struct Document {
    id: String,
    kind: String,
    title: String,
}

/// Идентификаторы `doc_anchor(id = "…")` файла. Комментарий разметки не
/// несёт: снятие идёт до поиска, и `doc_anchor` в тексте документа не
/// выдаётся за якорь кода (работа w-access-answers).
fn anchors_in(text: &str) -> Vec<String> {
    let text = record::strip_comments(text);
    let mut out = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("doc_anchor(") {
        let end = rest[at..]
            .find(")]")
            .map(|close| at + close)
            .unwrap_or(rest.len());
        let segment = &rest[at..end];
        if let Some(id_at) = segment.find("id = \"") {
            let after = &segment[id_at + 6..];
            if let Some(close) = after.find('"') {
                out.push(after[..close].to_owned());
            }
        }
        rest = &rest[end..];
    }
    out
}

/// Идентификатор якоря в ссылках Rust: `plan-ir` → `anchor::plan_ir`.
fn anchor_ident(anchor: &str) -> String {
    format!("anchor::{}", anchor.replace('-', "_"))
}

fn where_text(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    if anchors.is_empty() {
        return format!("no doc_anchor markup in {file}\n");
    }
    let mut out = format!("file {file}\nanchors: {}\n", anchors.join(", "));
    if !documents.is_empty() {
        out.push_str("documents:\n");
        for document in documents {
            out.push_str(&format!(
                "  {} {} — {}\n",
                document.kind, document.id, document.title
            ));
        }
    }
    if !works.is_empty() {
        out.push_str("works:\n");
        for work in works {
            out.push_str(&format!(
                "  {} [{}] — {}\n",
                work.id, work.state, work.title
            ));
        }
    }
    out
}

fn where_json(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    let mut out = format!(
        "{{\"schema\": \"dacc-where\", \"file\": {}, \"anchors\": [",
        json_string(file)
    );
    for (i, anchor) in anchors.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&json_string(anchor));
    }
    out.push_str("], \"documents\": [");
    for (i, document) in documents.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"kind\": {}, \"id\": {}, \"title\": {}}}",
            json_string(&document.kind),
            json_string(&document.id),
            json_string(&document.title)
        ));
    }
    out.push_str("], \"works\": [");
    for (i, work) in works.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"state\": {}, \"title\": {}}}",
            json_string(&work.id),
            json_string(&work.state),
            json_string(&work.title)
        ));
    }
    out.push_str("]}\n");
    out
}

/// Ответ where версионированным контрактом XML.
fn where_xml(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    let mut out = format!(
        "<where schema=\"dacc-where\" version=\"1\" file=\"{}\">\n",
        xml_string(file)
    );
    if anchors.is_empty() {
        out.push_str("  <no_markup/>\n</where>\n");
        return out;
    }
    for anchor in anchors {
        out.push_str(&format!("  <anchor id=\"{}\"/>\n", xml_string(anchor)));
    }
    for document in documents {
        out.push_str(&format!(
            "  <document kind=\"{}\" id=\"{}\" title=\"{}\"/>\n",
            xml_string(&document.kind),
            xml_string(&document.id),
            xml_string(&document.title)
        ));
    }
    for work in works {
        out.push_str(&format!(
            "  <work id=\"{}\" state=\"{}\" title=\"{}\"/>\n",
            xml_string(&work.id),
            xml_string(&work.state),
            xml_string(&work.title)
        ));
    }
    out.push_str("</where>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Разметка читается из кода: doc_anchor в комментарии — не якорь.
    #[test]
    fn where_ignores_comments_in_markup() {
        let text = "/// Пример: doc_anchor(id = \"fake-001\")\n#[doc_anchor(id = \"real-001\")]\npub struct S;\n";
        assert_eq!(anchors_in(text), vec!["real-001".to_owned()]);
    }
}
