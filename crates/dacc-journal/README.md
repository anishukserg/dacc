# dacc-journal

Журнал слоя работы [DACC](https://github.com/anishukserg/dacc): формат событий, автомат переходов и свёртка в состояние работ и срезов (решение 15).

- **Событие — файл** плоского подмножества TOML: `doc/journal/<wNNNN|sNNNN>/<ГГГГММДДTЧЧММССZ>-<событие>.toml`. События: `started`, `gate`, `landed`, `abandoned`, `closed`. Файл события не изменяется и не удаляется после коммита.
- **Автомат** — без событий → `started` → `landed` или `abandoned`; `landed` требует события `gate` с тем же деревом; закрытый срез не несёт незавершённой работы.
- **Без зависимостей.** Крейт подключают скан при сборке реестра и инструмент `cargo dacc` в хуке, и автомат переходов у них один.

Журнал самого DACC — [doc/journal](https://github.com/anishukserg/dacc/tree/master/doc/journal).

Лицензия — [Apache-2.0](https://github.com/anishukserg/dacc/blob/master/LICENSE).
