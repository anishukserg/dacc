# dacc-cli

Команда `cargo dacc` [DACC](https://github.com/anishukserg/dacc): правила коммитов, калитка, хуки git и журнал работы (решения 8, 14 и 15).

```text
cargo dacc commit -F <message> [--log <file>] [--timeout <seconds>] -- <paths…>
cargo dacc msg-check [--form-only] <message> | --range <range>
cargo dacc gate [--repo <directory>] [--journal-only] [<tree>]
cargo dacc hook pre-commit | commit-msg <message> | pre-push <remote> <url>
cargo dacc hooks install [--force]
cargo dacc work start | land | drop | state …
cargo dacc slice close <sNNNN>
cargo dacc journal hash [<revision>] | import --work <wNNNN> [--close-finished-slices]
```

- **Калитка** проверяет дерево коммита, а не рабочую копию: форматирование, clippy, тесты, атакующие doctest со сверкой кодов ошибок, документацию, сборку на минимальной версии Rust и политику зависимостей. Если проект задал свою команду, она выполняется отдельным шагом, и её вердикт входит в вердикт калитки.
- **Журнал**: команды `work` и `slice` пишут события и коммитят их; коммит, меняющий только журнал, переиспользует доказательство калитки для того же дерева.
- **Без внешних зависимостей**: хук собирает инструмент, и сборка не тянет граф крейтов.

Раскладку реестра, набор типов темы, её предел, ссылку на правила коммитов проекта, темы служебных коммитов, пол атакующих doctest и команды проекта — свою проверку сообщения и свой шаг калитки — задаёт `dacc.toml` в корне репозитория (решение 20). Без файла действуют соглашения DACC: `doc/` и `.githooks/`.

```bash
cargo install --locked --git https://github.com/anishukserg/dacc dacc-cli
```

Лицензия — [Apache-2.0](https://github.com/anishukserg/dacc/blob/master/LICENSE).
