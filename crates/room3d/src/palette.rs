//! Bảng màu của phòng (sRGB, màu phẳng, ấm, tương phản thấp) và cách đổi tên vật liệu trong
//! file `.glb` sang một vai trò của bảng. Đây là bước "đồng bộ phong cách" lúc chạy: mọi mô
//! hình CC0 mất vật liệu gốc và nhận màu của vai trò. Giữ đồng bộ với bảng trong README.

/// `0xRRGGBB` → sRGB 0..1.
const fn hex(c: u32) -> [f32; 3] {
    [
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    ]
}

/// Vai trò → màu sRGB (0..1).
pub const PALETTE: &[(&str, [f32; 3])] = &[
    ("wood_dark", hex(0x8A5A3C)),
    ("wood_light", hex(0xC99A6E)),
    ("wall", hex(0xE8D8C4)),
    ("paper", hex(0xF4EEDC)),
    ("gold", hex(0xE3B23C)),
    ("screen", hex(0x6F8FC4)),
    ("leaf", hex(0x6A994E)),
    ("curtain", hex(0xD9825F)),
    ("fur", hex(0xE3A15C)),
    ("fur_dark", hex(0xB06D3B)),
    ("ceramic", hex(0x81B29A)),
    ("metal", hex(0xA39A90)),
    ("ink", hex(0x3B3330)),
];

/// Quy tắc đổi tên, xét theo thứ tự: tên vật liệu (chữ thường) chứa chuỗi đầu thì ra vai trò
/// thứ hai. Tên vật liệu lấy từ các gói đang dùng (Kenney Furniture Kit, Quaternius Cat).
pub const RULES: &[(&str, &str)] = &[
    ("wood_light", "wood_light"),
    ("woodlight", "wood_light"),
    ("wood", "wood_dark"),
    ("eye_black", "ink"),
    ("metaldark", "ink"),
    ("eye_white", "paper"),
    ("carpetwhite", "paper"),
    ("paper", "paper"),
    ("lamp", "paper"),
    ("gold", "gold"),
    ("screen", "screen"),
    ("glass", "screen"),
    ("plant", "leaf"),
    ("leaf", "leaf"),
    ("carpet", "curtain"),
    ("curtain", "curtain"),
    ("cat_secondary", "fur_dark"),
    ("ears", "fur_dark"),
    ("cat_main", "fur"),
    ("fur", "fur"),
    ("ceramic", "ceramic"),
    ("mug", "ceramic"),
    ("metal", "metal"),
];

/// Vai trò của vật liệu tên `material_name` (không phân biệt hoa thường). Tên lạ ra `"wall"`.
pub fn role_for(material_name: &str) -> &'static str {
    let name = material_name.to_ascii_lowercase();
    RULES
        .iter()
        .find(|(needle, _)| name.contains(needle))
        .map_or("wall", |(_, role)| role)
}

/// Màu sRGB của vai trò `role`; vai trò lạ ra màu tường.
pub fn color_of(role: &str) -> [f32; 3] {
    PALETTE
        .iter()
        .find(|(n, _)| *n == role)
        .or_else(|| PALETTE.iter().find(|(n, _)| *n == "wall"))
        .map_or([1.0; 3], |(_, c)| *c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vai_tro_vat_lieu() {
        assert_eq!(role_for("Wood"), "wood_dark");
        assert_eq!(role_for("wood_light"), "wood_light");
        assert_eq!(role_for("abc"), "wall");
    }

    #[test]
    fn moi_vai_tro_co_trong_bang_mau() {
        let known = |r: &str| PALETTE.iter().any(|(n, _)| *n == r);
        for (_, role) in RULES {
            assert!(known(role), "vai trò {role} thiếu trong PALETTE");
        }
        assert!(known(role_for("abc")));
        // Tên vật liệu thật trong các file đang dùng (Kenney, Quaternius).
        for name in [
            "wood",
            "woodDark",
            "metal",
            "metalDark",
            "metalMedium",
            "lamp",
            "plant",
            "carpet",
            "carpetDarker",
            "carpetWhite",
            "_defaultMat",
            "Cat_Main",
            "Cat_Secondary",
            "Ears",
            "Eye_White",
            "Eye_Black",
        ] {
            assert!(known(role_for(name)), "{name} → {}", role_for(name));
        }
    }

    #[test]
    fn mau_trong_khoang_va_ten_khong_trung() {
        for (i, (n, c)) in PALETTE.iter().enumerate() {
            assert!(c.iter().all(|v| (0.0..=1.0).contains(v)), "{n}");
            assert!(PALETTE[..i].iter().all(|(m, _)| m != n), "{n} trùng");
        }
    }
}
