# Tình trạng dự án

File này trả lời ba câu: đang làm gì, ai cần quyết gì, làm gì tiếp. Đọc đầu tiên mỗi phiên. Cập nhật trước khi dừng ([ban-giao.md](playbooks/ban-giao.md)). `cargo xtask docs` kiểm ba mục `##` bắt buộc bên dưới có mặt.

**Giai đoạn:** nền móng. Đã có lõi trung lập, ba loại hoạt động (`bars`, `choice`, `section`) và vỏ web tối thiểu chạy ở <https://nam2341135.github.io/dyen/app/>.

**Cập nhật lần cuối:** 2026-11-27.

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
- **Nâng `actions/checkout` v4 → v7** (Node 24, hết cảnh báo Node 20). Nhánh `ci/checkout-v7`, PR #6, xếp chồng lên PR #5. Chọn v7 vì lỗi push từ `git worktree` của v6.0.0 (`pages.yml` dùng) đã sửa ở v6.0.1. Còn:
  - người duyệt và merge, sau PR #5;
  - sau khi merge vào `main`: xem lần chạy `pages` đầu tiên đẩy được lên `gh-pages`. PR không chạy được workflow này. Nếu đỏ ở bước "Đẩy lên gh-pages/app" thì đó là credential của checkout; đặt lại `@v4` rồi báo.
- **ADR 0014, lưu bài làm cục bộ** (trạng thái "Đề xuất"). Nhánh `docs/adr-luu-bai-lam`, PR #7, xếp chồng lên PR #6. Chỉ có tài liệu, chưa có code. Còn:
  - người duyệt ADR (xem "Chờ người quyết");
  - nếu chấp nhận: sửa ADR 0004 trỏ về ADR 0014, sửa định nghĩa `Attempt` trong glossary, rồi thêm việc "làm lưu bài làm" vào "Tiếp theo".

- **`bars`: khoá thao tác khi phiên kết thúc.** Nhánh `fix/bars-khoa-sau-nop`, PR #8, xếp chồng lên PR #7. Kết thúc = bấm "Xong" hoặc so sánh đúng; so sánh sai vẫn sửa được (giữ điểm một nửa và `recovered`). Các nút bị `disabled`, hình nhận class `locked`, và `Play::act_unless` bỏ qua đầu vào. Có test, đã thử ở 360 px, CI xanh. Còn: người duyệt và merge, sau PR #7.

- **Ghim Rust 1.100.0** trong `rust-toolchain.toml` (ADR 0007, bổ sung 2026-11-26). Nhánh `ci/ghim-rust`, PR #9, xếp chồng lên PR #8. CI xanh ở máy. Còn: người duyệt và merge, sau PR #8.

- **Bố cục vỏ web.** Nhánh `feat/bo-cuc-vo-web`, PR #10, xếp chồng lên PR #9. Người dùng bảo "cứ làm đi" nên làm thẳng, không qua bản phác. Đã xong:
  - thanh tiêu đề dính trên cùng, có nút "←" về danh mục (44 px);
  - danh mục dạng thẻ: số thứ tự trong gói, tiêu đề, đề bài cắt còn hai dòng;
  - khối kết quả: dấu ✓/!, "Đã đạt"/"Chưa đạt", điểm bên phải, mục "Nhận xét" chỉ hiện khi có phát hiện.

  Đã thử ở 360 px, CI xanh. Còn: người duyệt giao diện và merge, sau PR #9.

- **Ngôn ngữ thiết kế "giấy và mực", nút nổi** ([ADR 0015](adr/0015-ngon-ngu-thiet-ke-giay-muc.md), "Đề xuất"). Nhánh `feat/phong-cach-giay-muc`, PR #11, xếp chồng lên PR #10. Chủ dự án chọn hướng này sau khi xem ba bản phác. Đã xong:
  - bộ biến màu mới và bản tối (`prefers-color-scheme`);
  - màu mảnh `bars` theo Okabe–Ito;
  - tiêu đề serif dùng font có sẵn trên máy;
  - nút nổi, và lớp `primary` cho Nộp, Cắt, Xong.

  Đã thử ở 360 px, sáng và tối. Còn: người duyệt giao diện; chấp nhận ADR 0015; merge sau PR #10.

- **Trang chủ là phòng học 3D, mốc 1 (phòng + điều hướng).** Nhánh `feat/phong-hoc-3d`, đã commit đủ 8 task, chưa có PR (bản nháp mô tả PR đã viết, chờ chủ dự án cho mở). Thiết kế: [`docs/superpowers/specs/2026-11-27-phong-hoc-3d-design.md`](superpowers/specs/2026-11-27-phong-hoc-3d-design.md); kế hoạch: [`docs/superpowers/plans/2026-11-27-phong-hoc-3d.md`](superpowers/plans/2026-11-27-phong-hoc-3d.md). Đã xong:
  - crate thuần `room-model` (bảng `ROOM`, giao thức `ToRoom`/`FromRoom`);
  - vỏ web: định tuyến hash, danh sách chữ các khu, lớp phủ chuyển cảnh, menu "⋯" ("Phòng đơn giản", chất lượng);
  - `room3d` (Bevy): cảnh, mô hình CC0 (nguồn ở `crates/room3d/assets/NGUON.md`), trời theo giờ, mèo, bay camera, tự hạ chất lượng;
  - ảnh nền chờ `crates/web/room-still.webp`; `build.sh` build cả phòng (`--no-room` để bỏ qua); `pages.yml` đưa `dist/room/` lên Pages.

  Còn:
  - mở PR khi được phép;
  - workflow `pages` với phòng 3D chưa chạy lần nào (PR không chạy được). Sau khi merge: xem lần chạy đầu có đẩy được `app/room/` lên `gh-pages` không;
  - khung 3D đầu lần mở đầu tiên khoảng 11 s, vượt ngân sách 3 s; lần nạp lại 0,9 s (số đo ở `crates/web/README.md`);
  - các việc trong "Chờ người quyết" bên dưới.

  CI: xanh. Lệnh tiếp theo: `crates/web/build.sh` rồi `python -m http.server -d crates/web/dist 8091`.

## Chờ người quyết

Việc AI không tự làm được hoặc không được tự quyết.

- Duyệt và merge PR #5 (cấu trúc bàn giao), sau PR #4.
- Duyệt và merge PR #4, rồi thử vỏ web trên iPhone thật: vùng bấm, nút 44 px.
- Đo thời gian tải trên một điện thoại Android tầm trung. Đây là nợ từ spike ADR 0008; mới đo iPhone 12.
- ADR 0005: phần pháp lý (luật Việt Nam về dữ liệu trẻ em) chưa xác minh với người có chuyên môn.
- Duyệt [ADR 0014](adr/0014-luu-bai-lam-cuc-bo.md) (lưu bài làm cục bộ, ẩn danh). Cần quyết:
  - chấp nhận "bản chép cấu hình" thay cho "số phiên bản" của ADR 0004;
  - chỉ lưu phiên đã xong, giữ tối đa 500 bài làm.

  Chưa làm gì có lưu trữ trước khi ADR này được chấp nhận.
- Phòng học 3D (nhánh `feat/phong-hoc-3d`):
  - thử trên điện thoại thật: chạm đúng vật ở 360 px (canvas chất lượng thấp dùng CSS `transform`), thời gian khung đầu, xoay máy;
  - duyệt góc nhìn nhà ở 1280 px: bảng ghi chú và kệ huy chương nằm ngoài khung;
  - chấp nhận mèo đứng yên (khung đầu của `Idle`) khi giảm chuyển động, vì bộ mô hình không có cảnh ngủ.

## Tiếp theo

Theo thứ tự. "Làm tiếp" mà không có mục dở dang ở "Đang làm" thì làm mục đầu tiên ở đây, trên nhánh riêng `loại/tên-ngắn` ([ban-giao.md](playbooks/ban-giao.md)). Mỗi mục có tiêu chí xong.

1. **Ngân sách kích thước wasm** trong `cargo xtask` (ADR 0007).
   - Xong khi: CI đỏ nếu `web_bg.wasm` sau `wasm-opt` vượt ngưỡng ghi trong `crates/web/README.md`.
2. **Phòng 3D hiện khung đầu nhanh hơn.** Lần mở đầu tiên đang khoảng 11 s trên Chrome máy tính (ngân sách 3 s), phần lớn là biên dịch wasm 30 MB. Hướng thử: wasm nhỏ hơn (bỏ bớt tính năng Bevy, `wasm-opt -Oz`), biên dịch dạng luồng (`instantiateStreaming`), nạp phòng sau khi danh sách chữ đã hiện.
   - Xong khi: khung đầu lần mở đầu ≤ 3 s trên Chrome máy tính và đã đo trên một điện thoại Android tầm trung, số đo ghi trong `crates/room3d/README.md`.
3. **Phòng học 3D, mốc 2:** note tương tác, huy chương thật, đồ trong phòng đổi theo tiến độ. Cần ADR 0014 (lưu bài làm) được chấp nhận trước.
   - Xong khi: huy chương và note lấy từ bài làm đã lưu, có test, thử ở 360 px.
4. **Phòng học 3D, mốc 3:** trang trí phòng, kéo thả đồ.
   - Xong khi: có bản thiết kế riêng được duyệt, rồi làm theo nó.

## Nợ đã biết

- `bars` tự do: hai vết cắt người học đặt quá sát nhau cho vùng bấm < 24 px (ghi ở ADR 0013).
- GitHub Pages chỉ nén gzip, không brotli. Lượng tải thật lớn hơn số đo trong `crates/web/README.md`.
- Phòng 3D: khung đầu lần mở đầu tiên khoảng 11 s trên máy tính (ngân sách 3 s), chủ yếu do biên dịch 30 MB wasm; `room3d_bg.wasm` 4,57 MB brotli, còn dư khoảng 430 KB so với ngân sách 5 MB. Trên GitHub Pages (chỉ gzip) lượng tải lớn hơn.
- `README.md` của `crates/web` liệt kê kích thước đo tay. Chưa có máy kiểm (mục 1 ở "Tiếp theo").

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
