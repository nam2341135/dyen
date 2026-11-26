//! Chữ điểm ảnh tự vẽ cho các tờ note in sẵn trên tường (không dùng font ngoài, không
//! `bevy_text`). Mỗi chữ là một ô 5 × 12 điểm: 3 hàng dấu phía trên, 7 hàng thân, 2 hàng
//! đuôi (g, p, y). Dấu tiếng Việt ghép từ chữ gốc và dấu, chỉ đủ cho các câu dùng trong phòng
//! ([`NOTES`]); test kiểm mọi chữ trong đó đều vẽ được.

/// Bề ngang một chữ (điểm), chưa kể khoảng cách 1 điểm giữa hai chữ.
pub const GLYPH_W: u32 = 5;
/// Chiều cao một ô chữ (điểm): 3 hàng dấu + 7 hàng thân + 2 hàng đuôi.
pub const GLYPH_H: u32 = 12;
/// Số hàng dấu phía trên thân chữ.
const TOP: i32 = 3;

/// Nội dung in sẵn trên tường: hai tờ note, và tấm vé trên kệ huy chương. Mỗi mục là các dòng.
pub const NOTES: [&[&str]; 3] = [
    &["Chào mừng", "tới phòng", "học!"],
    &["Bấm vào", "vở để", "ôn tập"],
    &["Ngày đầu", "tới lớp"],
];

/// Thân chữ gốc: 7 hàng thân + 2 hàng đuôi, mỗi hàng 5 điểm (`#` là mực).
fn base(c: char) -> Option<[&'static str; 9]> {
    let g = match c {
        ' ' => [
            ".....", ".....", ".....", ".....", ".....", ".....", ".....", ".....", ".....",
        ],
        '!' => [
            "..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#..", ".....", ".....",
        ],
        'B' => [
            "####.", "#...#", "#...#", "####.", "#...#", "#...#", "####.", ".....", ".....",
        ],
        'C' => [
            ".###.", "#...#", "#....", "#....", "#....", "#...#", ".###.", ".....", ".....",
        ],
        'N' => [
            "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#", ".....", ".....",
        ],
        'a' => [
            ".....", ".....", ".###.", "....#", ".####", "#...#", ".####", ".....", ".....",
        ],
        'c' => [
            ".....", ".....", ".###.", "#....", "#....", "#...#", ".###.", ".....", ".....",
        ],
        'd' => [
            "....#", "....#", ".####", "#...#", "#...#", "#...#", ".####", ".....", ".....",
        ],
        'đ' => [
            "....#", "..###", ".####", "#...#", "#...#", "#...#", ".####", ".....", ".....",
        ],
        'e' => [
            ".....", ".....", ".###.", "#...#", "#####", "#....", ".###.", ".....", ".....",
        ],
        'g' => [
            ".....", ".....", ".####", "#...#", "#...#", "#...#", ".####", "....#", ".###.",
        ],
        'h' => [
            "#....", "#....", "####.", "#...#", "#...#", "#...#", "#...#", ".....", ".....",
        ],
        'i' => [
            ".....", "..#..", ".....", ".##..", "..#..", "..#..", ".###.", ".....", ".....",
        ],
        'l' => [
            ".##..", "..#..", "..#..", "..#..", "..#..", "..#..", ".###.", ".....", ".....",
        ],
        'm' => [
            ".....", ".....", "##.#.", "#.#.#", "#.#.#", "#.#.#", "#.#.#", ".....", ".....",
        ],
        'n' => [
            ".....", ".....", "####.", "#...#", "#...#", "#...#", "#...#", ".....", ".....",
        ],
        'o' => [
            ".....", ".....", ".###.", "#...#", "#...#", "#...#", ".###.", ".....", ".....",
        ],
        'p' => [
            ".....", ".....", "####.", "#...#", "#...#", "#...#", "####.", "#....", "#....",
        ],
        't' => [
            ".....", ".#...", "####.", ".#...", ".#...", ".#..#", "..##.", ".....", ".....",
        ],
        'u' => [
            ".....", ".....", "#...#", "#...#", "#...#", "#...#", ".####", ".....", ".....",
        ],
        'v' => [
            ".....", ".....", "#...#", "#...#", "#...#", ".#.#.", "..#..", ".....", ".....",
        ],
        'y' => [
            ".....", ".....", "#...#", "#...#", "#...#", "#...#", ".####", "....#", ".###.",
        ],
        _ => return None,
    };
    Some(g)
}

/// Mũ (â, ê, ô).
const HAT: [&str; 2] = ["..#..", ".#.#."];

/// Dấu thanh.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tone {
    None,
    Grave,
    Acute,
    Hook,
    DotBelow,
}

/// Hình dấu thanh phía trên (dòng cuối nằm sát trên chữ hoặc trên mũ).
fn tone_rows(t: Tone) -> &'static [&'static str] {
    match t {
        Tone::Grave => &[".#...", "..#.."],
        Tone::Acute => &["...#.", "..#.."],
        Tone::Hook => &[".##..", "...#.", "..#.."],
        Tone::None | Tone::DotBelow => &[],
    }
}

/// Tách chữ có dấu thành (chữ gốc, mũ, móc ơ/ư, dấu thanh). Chỉ các chữ cần cho [`NOTES`].
fn split(c: char) -> Option<(char, bool, bool, Tone)> {
    use Tone::*;
    let r = match c {
        'à' => ('a', false, false, Grave),
        'ấ' => ('a', true, false, Acute),
        'ầ' => ('a', true, false, Grave),
        'ậ' => ('a', true, false, DotBelow),
        'ể' => ('e', true, false, Hook),
        'ô' => ('o', true, false, None),
        'ò' => ('o', false, false, Grave),
        'ọ' => ('o', false, false, DotBelow),
        'ớ' => ('o', false, true, Acute),
        'ở' => ('o', false, true, Hook),
        'ừ' => ('u', false, true, Grave),
        other => (other, false, false, None),
    };
    Some(r)
}

/// Các điểm mực của chữ `c` trong ô 5 × 12, toạ độ (cột, hàng) từ góc trên trái. `None` nếu
/// không vẽ được chữ này.
pub fn glyph(c: char) -> Option<Vec<(u32, u32)>> {
    let (b, hat, horn, tone) = split(c)?;
    let body = base(b)?;
    let mut px: Vec<(i32, i32)> = Vec::new();
    let put = |rows: &[&str], top: i32, px: &mut Vec<(i32, i32)>| {
        for (r, line) in rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == '#' {
                    px.push((col as i32, top + r as i32));
                }
            }
        }
    };
    put(&body, TOP, &mut px);
    // Chữ thường: thân bắt đầu ở hàng 2; mũ nằm ở hàng 0–1 của thân, dấu thanh ở trên nữa.
    let mut above = TOP + 2;
    if hat {
        put(&HAT, TOP, &mut px);
        above = TOP;
    }
    if horn {
        px.push((4, TOP + 1));
        px.push((4, TOP + 2));
    }
    let t = tone_rows(tone);
    put(t, above - t.len() as i32, &mut px);
    if tone == Tone::DotBelow {
        px.push((2, TOP + 8));
    }
    px.sort_unstable();
    px.dedup();
    // Mọi hàng đều ≥ 0 (dấu cao nhất là dấu hỏi trên mũ, bắt đầu ở hàng 0).
    Some(
        px.into_iter()
            .filter_map(|(x, y)| Some((u32::try_from(x).ok()?, u32::try_from(y).ok()?)))
            .collect(),
    )
}

/// Bề ngang (điểm, chưa phóng) của một dòng.
pub fn line_width(line: &str) -> u32 {
    let n = line.chars().count() as u32;
    (n * (GLYPH_W + 1)).saturating_sub(1)
}

/// Ảnh RGBA `w × h`: nền `bg`, các dòng chữ màu `ink` ở giữa, mỗi điểm phóng `scale` lần.
/// Chữ không vẽ được thì bỏ qua (test giữ cho điều đó không xảy ra với [`NOTES`]).
pub fn text_rgba(lines: &[&str], w: u32, h: u32, scale: u32, ink: [u8; 4], bg: [u8; 4]) -> Vec<u8> {
    let mut img = blank(w, h, bg);
    let total_h = lines.len() as u32 * GLYPH_H * scale;
    let y0 = h.saturating_sub(total_h) / 2;
    for (i, line) in lines.iter().enumerate() {
        let x0 = w.saturating_sub(line_width(line) * scale) / 2;
        let top = y0 + i as u32 * GLYPH_H * scale;
        for (k, c) in line.chars().enumerate() {
            let left = x0 + k as u32 * (GLYPH_W + 1) * scale;
            for (gx, gy) in glyph(c).unwrap_or_default() {
                fill(
                    &mut img,
                    (w, h),
                    left + gx * scale,
                    top + gy * scale,
                    scale,
                    ink,
                );
            }
        }
    }
    img
}

/// Ảnh RGBA `size × size`: một ngôi sao năm cánh màu `ink` trên nền `bg` (tờ note vẽ sao).
pub fn star_rgba(size: u32, ink: [u8; 4], bg: [u8; 4]) -> Vec<u8> {
    let mut img = blank(size, size, bg);
    let c = size as f32 / 2.0;
    let (outer, inner) = (size as f32 * 0.38, size as f32 * 0.16);
    // Mười đỉnh xen kẽ ngoài/trong, đỉnh đầu chĩa lên.
    let pts: Vec<(f32, f32)> = (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { outer } else { inner };
            let a = std::f32::consts::PI * (i as f32 / 5.0 - 0.5);
            (c + r * a.cos(), c + r * a.sin())
        })
        .collect();
    for y in 0..size {
        for x in 0..size {
            if inside(&pts, x as f32 + 0.5, y as f32 + 0.5) {
                fill(&mut img, (size, size), x, y, 1, ink);
            }
        }
    }
    img
}

/// Điểm (x, y) trong đa giác (luật chẵn lẻ).
fn inside(poly: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut odd = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            odd = !odd;
        }
        j = i;
    }
    odd
}

fn blank(w: u32, h: u32, bg: [u8; 4]) -> Vec<u8> {
    bg.repeat((w * h) as usize)
}

/// Tô ô vuông `n × n` bắt đầu ở (x, y) trong ảnh cỡ `(w, h)`; phần tràn ra ngoài bị bỏ.
fn fill(img: &mut [u8], (w, h): (u32, u32), x: u32, y: u32, n: u32, rgba: [u8; 4]) {
    for yy in y..(y + n).min(h) {
        for xx in x..(x + n).min(w) {
            let i = ((yy * w + xx) * 4) as usize;
            img[i..i + 4].copy_from_slice(&rgba);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INK: [u8; 4] = [0, 0, 0, 255];
    const BG: [u8; 4] = [255, 255, 255, 255];

    #[test]
    fn moi_chu_trong_note_deu_ve_duoc() {
        for note in NOTES {
            for line in note {
                for c in line.chars() {
                    let g = glyph(c).unwrap_or_else(|| panic!("thiếu chữ {c:?}"));
                    assert!(c == ' ' || !g.is_empty(), "{c:?} trống");
                    assert!(g.iter().all(|&(x, y)| x < GLYPH_W && y < GLYPH_H), "{c:?}");
                }
            }
        }
    }

    #[test]
    fn chu_co_dau_khac_chu_goc() {
        let a = glyph('a');
        for c in ['à', 'ấ', 'ầ', 'ậ'] {
            assert_ne!(glyph(c), a, "{c}");
        }
        assert_ne!(glyph('ớ'), glyph('ở'));
        assert_ne!(glyph('ô'), glyph('o'));
        assert_eq!(glyph('Z'), None);
    }

    #[test]
    fn note_vua_anh_128_phong_2() {
        for note in NOTES {
            assert!(note.len() as u32 * GLYPH_H * 2 <= 128);
            for line in note {
                assert!(line_width(line) * 2 <= 128 - 8, "{line}");
            }
        }
    }

    #[test]
    fn anh_chu_co_muc_va_dung_kich_thuoc() {
        let img = text_rgba(NOTES[0], 128, 96, 2, INK, BG);
        assert_eq!(img.len(), 128 * 96 * 4);
        let ink = img.chunks(4).filter(|p| *p == INK).count();
        assert!(ink > 200, "{ink}");
    }

    #[test]
    fn ngoi_sao_o_giua_anh() {
        let img = star_rgba(64, INK, BG);
        let at = |x: usize, y: usize| &img[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
        assert_eq!(at(32, 32), INK);
        assert_eq!(at(1, 1), BG);
    }
}
