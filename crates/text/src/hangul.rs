//! Hangul syllable composition (NFD -> NFC) for conjoining jamo sequences.
//!
//! On macOS, file systems (HFS+/APFS) and native file dialogs decompose Korean
//! text into NFD conjoining jamo sequences (Choseong U+1100..U+1112, Jungseong
//! U+1161..U+1175, Jongseong U+11A8..U+11C2). This module composes them into
//! standard precomposed Hangul syllables (U+AC00..U+D7A3) following Unicode
//! Standard §3.12 (Conjoining Jamo Behavior).

const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;

/// Composes NFD conjoining jamos in `s` into NFC precomposed Hangul syllables.
/// Non-Hangul characters and already-composed syllables are preserved as-is.
pub fn compose_hangul(s: &str) -> String {
    // Fast path: if there are no leading conjoining jamos, no composition is needed.
    if !s.chars().any(|c| (L_BASE..L_BASE + L_COUNT).contains(&(c as u32))) {
        return s.to_string();
    }

    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let len = chars.len();

    while i < len {
        let c = chars[i];
        let code = c as u32;

        if (L_BASE..L_BASE + L_COUNT).contains(&code) && i + 1 < len {
            let next_code = chars[i + 1] as u32;
            if (V_BASE..V_BASE + V_COUNT).contains(&next_code) {
                let l_idx = code - L_BASE;
                let v_idx = next_code - V_BASE;
                let lv_syllable = S_BASE + (l_idx * V_COUNT + v_idx) * T_COUNT;
                i += 2;

                if i < len {
                    let trail_code = chars[i] as u32;
                    if (T_BASE + 1..T_BASE + T_COUNT).contains(&trail_code) {
                        let t_idx = trail_code - T_BASE;
                        if let Some(composed) = char::from_u32(lv_syllable + t_idx) {
                            out.push(composed);
                            i += 1;
                            continue;
                        }
                    }
                }

                if let Some(composed) = char::from_u32(lv_syllable) {
                    out.push(composed);
                    continue;
                }
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_nfc_remain_unchanged() {
        assert_eq!(compose_hangul("hello_world.psd"), "hello_world.psd");
        assert_eq!(compose_hangul("한국어_파일.psd"), "한국어_파일.psd");
        assert_eq!(compose_hangul("/Users/test/사진/폴더/이미지.png"), "/Users/test/사진/폴더/이미지.png");
    }

    #[test]
    fn nfd_jamo_composes_to_nfc() {
        // "한" decomposed: U+1112 (ㅎ) + U+1161 (ㅏ) + U+11AB (ㄴ)
        let nfd_han = "\u{1112}\u{1161}\u{11AB}";
        assert_eq!(compose_hangul(nfd_han), "한");

        // "가" decomposed: U+1100 (ㄱ) + U+1161 (ㅏ)
        let nfd_ga = "\u{1100}\u{1161}";
        assert_eq!(compose_hangul(nfd_ga), "가");

        // "한국어" decomposed
        let nfd_korean = "\u{1112}\u{1161}\u{11AB}\u{1100}\u{116E}\u{11A8}\u{110B}\u{1165}";
        assert_eq!(compose_hangul(nfd_korean), "한국어");

        // Complex filename with path separators
        let nfd_path = format!("/Volumes/Data/{nfd_korean}/test_{nfd_ga}.psd");
        assert_eq!(compose_hangul(&nfd_path), "/Volumes/Data/한국어/test_가.psd");
    }

    #[test]
    fn incomplete_jamo_sequences_are_preserved() {
        // Lone choseong followed by ASCII
        assert_eq!(compose_hangul("\u{1100}abc"), "\u{1100}abc");
        // Lone jungseong
        assert_eq!(compose_hangul("\u{1161}test"), "\u{1161}test");
    }
}
