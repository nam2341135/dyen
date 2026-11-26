@AGENTS.md

## Riêng cho Claude Code

- Hướng dẫn chung nằm ở `AGENTS.md` (được nhập ở trên). Sửa ở đó, không chép sang đây. `cargo xtask docs` kiểm dòng nhập này.
- Auto memory chỉ nằm trên một máy. Điều gì người sau cần biết thì ghi vào repo (`docs/STATUS.md`, ADR, README), không chỉ ghi vào memory.
- Ledger của superpowers (`.superpowers/`) bị `.gitignore`. Các "Ruling" trong đó phải chép vào mô tả PR trước khi xong việc.
- Thử vỏ web: cấu hình `web` trong `.claude/launch.json` phục vụ `crates/web/dist` ở cổng 8091. Thử ở bề rộng 360 px (ADR 0013).
