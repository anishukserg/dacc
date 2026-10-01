//! Стабильные коды причин отказа (решение 22).
//!
//! Код — контракт, а не фраза: перевод и переформулировка текста его не меняют.
//! Код печатается рядом с текстом человеку и входит в машинный вывод. Каждый
//! код уникален и несёт ровно один смысл; реестр [`ALL`] — единственный
//! перечень, и тест отвергает повторное использование кода.

/// Код причины отказа: kebab-case, уникальный по всему инструменту.
pub type Code = &'static str;

// Команда commit.
pub const TIMEOUT_SECONDS: Code = "timeout-not-seconds";
pub const FORMAT_NOT_UTF8: Code = "format-value-not-utf8";
pub const FORMAT_CHOICE: Code = "format-invalid";
pub const FLAG_NEEDS_VALUE: Code = "flag-needs-value";
pub const UNKNOWN_ARGUMENT: Code = "unknown-argument";
pub const MESSAGE_FILE_REQUIRED: Code = "message-file-required";
pub const MESSAGE_CONFIG: Code = "message-config-invalid";
pub const NO_PATHS_LISTED: Code = "no-paths-listed";
pub const REPO_DISCOVER: Code = "repo-not-discovered";
pub const MESSAGE_REFUSED: Code = "message-refused";
pub const LOCK_NOT_ACQUIRED: Code = "lock-not-acquired";
pub const GIT_ADD_FAILED: Code = "git-add-failed";
pub const NOTHING_TO_COMMIT: Code = "nothing-to-commit";
pub const COMMIT_FAILED: Code = "commit-failed";
pub const HEAD_NOT_READ: Code = "head-not-read";
pub const LOG_NOT_OPEN: Code = "log-not-open";

// Команды work и slice.
pub const TRAILER_NEEDS_VALUE: Code = "trailer-needs-value";
pub const TRAILER_KEY_VALUE: Code = "trailer-key-value-form";
pub const WORK_NOT_IN_PLAN: Code = "work-not-in-plan";
pub const WORK_STAGE: Code = "work-stage-invalid";
pub const REVISION_NOT_COMMIT: Code = "revision-not-commit";
pub const COMMIT_NOT_ANCESTOR: Code = "commit-not-in-history";
pub const COMMIT_NOT_BASED: Code = "commit-not-based-on-work";
pub const TREE_NOT_READ: Code = "tree-not-read";
pub const NO_PROOF: Code = "no-proof";
pub const FULL_TIER_FAILED: Code = "full-tier-failed";
pub const REASON_REQUIRED: Code = "reason-required";
pub const PROOF_RED_BEFORE: Code = "red-before-required";
pub const FILE_NOT_READ: Code = "file-not-read";
pub const PROOF_SUBJECT_REQUIRED: Code = "proof-subject-required";
pub const PROOF_REPEATED: Code = "proof-kind-repeated";
pub const SLICE_ID: Code = "not-slice-id";
pub const SLICE_NO_WORKS: Code = "slice-has-no-works";
pub const SLICE_ALREADY_CLOSED: Code = "slice-already-closed";
pub const SLICE_UNFINISHED: Code = "slice-unfinished";
pub const JOURNAL_NOT_READ: Code = "journal-not-read";
pub const JOURNAL_NOT_FOLD: Code = "journal-not-folding";
pub const PLAN_NOT_READ: Code = "plan-not-read";
pub const SUBSYSTEM_NOT_READ: Code = "subsystem-not-read";
pub const NOT_WORK_ID: Code = "not-work-id";
pub const EVENT_NOT_WRITTEN: Code = "event-not-written";
pub const COMMIT_MESSAGE_NOT_WRITTEN: Code = "commit-message-not-written";
pub const NOTHING_TO_IMPORT: Code = "nothing-to-import";
pub const USAGE: Code = "usage";

// Проверка сообщения.
pub const SUBJECT_FORM: Code = "subject-form-invalid";
pub const TYPE_NOT_IN_SET: Code = "type-not-in-set";
pub const SCOPE_NOT_AXIS: Code = "scope-not-axis";
pub const SUBJECT_PERIOD: Code = "subject-ends-with-period";
pub const SUBJECT_TOO_LONG: Code = "subject-too-long";
pub const SUBJECT_BLANK_LINE: Code = "subject-blank-line-missing";
pub const BASIS_TRAILER_MISSING: Code = "basis-trailer-missing";
pub const TRAILER_FORM_INVALID: Code = "trailer-form-invalid";
pub const WORK_NOT_IN_TREE: Code = "work-not-in-tree";
pub const SLICE_NOT_IN_TREE: Code = "slice-not-in-tree";
pub const DELEGATED_REFUSED: Code = "subject-refused-by-command";

// Калитка.
pub const STEP_FAILED: Code = "step-failed";
pub const START_ERROR: Code = "start-error";

/// Реестр всех кодов с их смыслом — для проверки уникальности. Каждый код
/// встречается ровно один раз; повторное использование кода с другим смыслом
/// ломает тест. В рабочем коде не читается — его единственный потребитель тест,
/// поэтому мёртвый код для сборки без тестов разрешён.
#[allow(dead_code)]
pub const ALL: &[(&str, &str)] = &[
    (TIMEOUT_SECONDS, "commit: --timeout needs seconds"),
    (FORMAT_NOT_UTF8, "commit: --format value is not UTF-8"),
    (FORMAT_CHOICE, "commit/gate: --format needs json or text"),
    (FLAG_NEEDS_VALUE, "commit: a flag needs a value"),
    (UNKNOWN_ARGUMENT, "commit: unknown argument"),
    (MESSAGE_FILE_REQUIRED, "a readable message file is required"),
    (
        MESSAGE_CONFIG,
        "commit: the message rules cannot be read from the tree",
    ),
    (NO_PATHS_LISTED, "commit: no paths listed"),
    (REPO_DISCOVER, "commit: repository not discovered"),
    (MESSAGE_REFUSED, "the message is refused by the form rules"),
    (
        LOCK_NOT_ACQUIRED,
        "commit: the commit lock was not acquired",
    ),
    (GIT_ADD_FAILED, "commit: git add failed on the listed paths"),
    (
        NOTHING_TO_COMMIT,
        "commit: nothing to commit in the listed paths",
    ),
    (COMMIT_FAILED, "commit: git commit failed"),
    (
        HEAD_NOT_READ,
        "commit: HEAD cannot be read after the commit",
    ),
    (LOG_NOT_OPEN, "commit: the log file cannot be opened"),
    (TRAILER_NEEDS_VALUE, "work: --trailer needs a trailer line"),
    (
        TRAILER_KEY_VALUE,
        "work: a trailer is not of the form Key: value",
    ),
    (WORK_NOT_IN_PLAN, "work: the work is not in the plan"),
    (WORK_STAGE, "work: the work is already in another stage"),
    (REVISION_NOT_COMMIT, "work: the revision is not a commit"),
    (
        COMMIT_NOT_ANCESTOR,
        "work: the commit is not in the history of HEAD",
    ),
    (
        COMMIT_NOT_BASED,
        "work: the commit has no trailer for the work",
    ),
    (TREE_NOT_READ, "work: the tree of the commit cannot be read"),
    (NO_PROOF, "work: no gate proof for the tree"),
    (
        FULL_TIER_FAILED,
        "work: the full gate tier did not pass on the tree",
    ),
    (REASON_REQUIRED, "work: --reason <reason> is required"),
    (FILE_NOT_READ, "where: the file cannot be read"),
    (
        PROOF_RED_BEFORE,
        "work: a defect fix or an irreversible change lands only with a RedBefore proof",
    ),
    (
        PROOF_SUBJECT_REQUIRED,
        "work: a proof kind needs a non-empty subject",
    ),
    (PROOF_REPEATED, "work: a proof kind is stated once"),
    (SLICE_ID, "slice: the id is not of the form s-slug"),
    (SLICE_NO_WORKS, "slice: the slice has no works"),
    (SLICE_ALREADY_CLOSED, "slice: the slice is already closed"),
    (SLICE_UNFINISHED, "slice: the slice has unfinished works"),
    (JOURNAL_NOT_READ, "work: the journal cannot be read"),
    (JOURNAL_NOT_FOLD, "work: the journal does not fold"),
    (PLAN_NOT_READ, "work: the plan cannot be read"),
    (
        SUBSYSTEM_NOT_READ,
        "work: the subsystem taxon of the record cannot be read",
    ),
    (NOT_WORK_ID, "work: the id is not of the form w-slug"),
    (
        EVENT_NOT_WRITTEN,
        "work: the journal event cannot be written",
    ),
    (
        COMMIT_MESSAGE_NOT_WRITTEN,
        "work: the commit message cannot be written",
    ),
    (NOTHING_TO_IMPORT, "journal import: nothing to import"),
    (USAGE, "work: invalid arguments"),
    (
        SUBJECT_FORM,
        "message: subject is not in the form [TYPE](scope): summary",
    ),
    (TYPE_NOT_IN_SET, "message: the type is not in the set"),
    (
        SCOPE_NOT_AXIS,
        "message: the scope is not a subsystem axis value",
    ),
    (SUBJECT_PERIOD, "message: the subject ends with a period"),
    (SUBJECT_TOO_LONG, "message: the subject is too long"),
    (
        SUBJECT_BLANK_LINE,
        "message: a blank line must follow the subject",
    ),
    (
        BASIS_TRAILER_MISSING,
        "message: the basis trailer is missing",
    ),
    (
        TRAILER_FORM_INVALID,
        "message: a trailer is not in the form slug",
    ),
    (
        WORK_NOT_IN_TREE,
        "message: the work is not in the commit tree",
    ),
    (
        SLICE_NOT_IN_TREE,
        "message: the slice is not in the commit tree",
    ),
    (
        DELEGATED_REFUSED,
        "message: the subject is refused by the delegated command",
    ),
    (STEP_FAILED, "gate: a step failed"),
    (START_ERROR, "gate: the gate cannot start"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Реестр не переиспользует код с другим смыслом: каждый код один, и ни
    /// один код не остался вне реестра (константа без записи — тоже отказ).
    #[test]
    fn every_code_is_registered_once() {
        let codes: Vec<&str> = ALL.iter().map(|(code, _)| *code).collect();
        let unique: BTreeSet<&str> = codes.iter().copied().collect();
        assert_eq!(unique.len(), codes.len(), "a code is used more than once");
    }
}
