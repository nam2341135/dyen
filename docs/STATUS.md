# Tình trạng dự án

File này trả lời ba câu: đang làm gì, ai cần quyết gì, làm gì tiếp. Đọc đầu tiên mỗi phiên. Cập nhật trước khi dừng ([ban-giao.md](playbooks/ban-giao.md)). `cargo xtask docs` kiểm ba mục `##` bắt buộc bên dưới có mặt.

**Giai đoạn:** nền móng. Đã có lõi trung lập, ba loại hoạt động (`bars`, `choice`, `section`) và vỏ web tối thiểu chạy ở <https://nam2341135.github.io/dyen/app/>.

**Cập nhật lần cuối:** 2026-11-26.

## Đang làm

Mỗi mục: nhánh / PR, đã xong gì, còn gì, lệnh tiếp theo.

- **ADR 0013, quy ước bộ hiển thị.** Nhánh `docs/adr-renderer-rules`, PR #4 đang mở. Code và ADR đã xong, CI xanh, đã đo ở 360 px. Còn: người duyệt và merge.
- **Cấu trúc bàn giao.** Nhánh `docs/handoff`, tách từ `docs/adr-renderer-rules` (merge PR #4 trước). Đã xong:
  - `AGENTS.md` là nguồn hướng dẫn duy nhất, `CLAUDE.md` chỉ nhập nó;
  - file này;
  - [`docs/playbooks/`](playbooks/README.md);
  - `cargo xtask docs` trong `ci`;
  - mẫu PR.

  CI xanh, PR #5 đang mở (xếp chồng lên PR #4). Còn: người duyệt và merge, sau PR #4.

## Chờ người quyết

Việc AI không tự làm được hoặc không được tự quyết.

- Duyệt và merge PR #5 (cấu trúc bàn giao), sau PR #4.
- Duyệt và merge PR #4, rồi thử vỏ web trên iPhone thật: vùng bấm, nút 44 px.
- Đo thời gian tải trên một điện thoại Android tầm trung. Đây là nợ từ spike ADR 0008; mới đo iPhone 12.
- ADR 0005: phần pháp lý (luật Việt Nam về dữ liệu trẻ em) chưa xác minh với người có chuyên môn.
- Có lưu bài làm (`Attempt`) không, và lưu ở đâu. Cần một ADR (mục 3 ở "Tiếp theo") trước khi làm bất cứ gì có lưu trữ.

## Tiếp theo

Theo thứ tự. "Làm tiếp" mà không có mục dở dang ở "Đang làm" thì làm mục đầu tiên ở đây, trên nhánh riêng `loại/tên-ngắn` ([ban-giao.md](playbooks/ban-giao.md)). Mỗi mục có tiêu chí xong.

1. **Nâng `actions/checkout@v4` lên v5** trong `.github/workflows/ci.yml` và `pages.yml`. GitHub báo Node 20 sắp bị bỏ.
   - Xong khi: workflow chạy xanh, không còn cảnh báo Node 20.
2. **`bars`: khoá thao tác sau khi đã nộp.** Sau "Xong"/"So sánh", các nút và vùng bấm vẫn trông như bấm được dù phiên đã kết thúc.
   - Xong khi: vỏ web không phát `Input` khi `Session` đã xong.
   - Có test đơn vị.
   - ADR 0013 vẫn đúng: không khoá vì animation, chỉ khoá vì phiên đã kết thúc.
3. **Viết ADR lưu bài làm**: cục bộ, ẩn danh trước, theo ADR 0005.
   - Xong khi: ADR ở trạng thái "Đề xuất", nằm trong bảng ở `docs/README.md`, và mục tương ứng ở "Chờ người quyết" trỏ tới nó.
4. **Bố cục và giao diện vỏ web.** Gồm trang danh mục, thanh tiêu đề, trang kết quả. Chỉ sửa stylesheet và cấu trúc theo ADR 0013.
   - Xong khi: có bản thiết kế được duyệt, thử ở 360 px.
5. **Ghim phiên bản Rust chính xác** trong `rust-toolchain.toml`. Đây là nợ ghi ở ADR 0007, đến hạn vì đã có CI.
   - Xong khi: CI và máy cục bộ dùng cùng một bản, ADR 0007 ghi đã làm.
6. **Ngân sách kích thước wasm** trong `cargo xtask` (ADR 0007).
   - Xong khi: CI đỏ nếu `web_bg.wasm` sau `wasm-opt` vượt ngưỡng ghi trong `crates/web/README.md`.

## Nợ đã biết

- `bars` tự do: hai vết cắt người học đặt quá sát nhau cho vùng bấm < 24 px (ghi ở ADR 0013).
- GitHub Pages chỉ nén gzip, không brotli. Lượng tải thật lớn hơn số đo trong `crates/web/README.md`.
- `README.md` của `crates/web` liệt kê kích thước đo tay. Chưa có máy kiểm (mục 6 ở trên).

## Đã xong gần đây

| Việc | PR / commit |
|---|---|
| Lõi trung lập: `kernel`, `kind-bars`, `registry` (ADR 0010) | trước PR #1 |
| Spike Leptos + Bevy, chốt ADR 0008 | trước PR #1 |
| `kind-choice` (sinh ra ADR 0011) | trước PR #1 |
| `kind-section` cùng `lib-exact` (ADR 0012) | trước PR #1 |
| CI trên GitHub Actions | 4303917 |
| Vỏ web tối thiểu (Leptos CSR), wasm 188 KB brotli | PR #1 |
| Phép chiếu `section` cho thấy rõ thiết diện lục giác | PR #2 |
| Vỏ web lên GitHub Pages | PR #3 |
