// The algorithm is from https://github.com/nayuki/QR-Code-generator/pull/40/

use alloc::string::String;
use alloc::vec::Vec;

use crate::qrcode_generator::qrcodegen::{QrCodeEcc, QrSegment, QrSegmentMode, Version};

#[cfg(feature = "kanji")]
use crate::qrcode_generator::qrcodegen::BitBuffer;

#[cfg(feature = "kanji")]
const MODE_TYPES: [QrSegmentMode; 4] = [
    QrSegmentMode::Byte,
    QrSegmentMode::Alphanumeric,
    QrSegmentMode::Numeric,
    QrSegmentMode::Kanji,
];
#[cfg(not(feature = "kanji"))]
const MODE_TYPES: [QrSegmentMode; 3] =
    [QrSegmentMode::Byte, QrSegmentMode::Alphanumeric, QrSegmentMode::Numeric];

#[cfg(feature = "kanji")]
const NUM_MODES: usize = 4;
#[cfg(not(feature = "kanji"))]
const NUM_MODES: usize = 3;

// The set of all legal characters in alphanumeric mode,
// where each character value maps to the index in the string.
static ALPHANUMERIC_CHARSET: [char; 45] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$',
    '%', '*', '+', '-', '.', '/', ':',
];

static ECC_CODEWORDS_PER_BLOCK: [[i8; 41]; 4] = [
    // Version: (note that index 0 is for padding, and is set to an illegal value)
    //0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40    Error correction level
    [
        -1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ], // Low
    [
        -1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ], // Medium
    [
        -1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ], // Quartile
    [
        -1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ], // High
];

static NUM_ERROR_CORRECTION_BLOCKS: [[i8; 41]; 4] = [
    // Version: (note that index 0 is for padding, and is set to an illegal value)
    //0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40    Error correction level
    [
        -1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12,
        13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ], // Low
    [
        -1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ], // Medium
    [
        -1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27,
        29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ], // Quartile
    [
        -1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ], // High
];

#[cfg(feature = "kanji")]
pub(crate) static UNICODE_TO_QR_KANJI: [i16; 1 << 16] =
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/unicode_to_qr_kanji.json"));

/// Returns a list of zero or more segments to represent the specified Unicode text string.
pub(crate) fn make_segments_optimally(
    code_points: &[char],
    ecc: QrCodeEcc,
    min_version: Version,
    max_version: Version,
) -> Option<Vec<QrSegment>> {
    let min_version = min_version.value();
    let max_version = max_version.value();

    // Check arguments
    // if min_version > max_version {
    //     return None;
    // }

    // Iterate through version numbers, and make tentative segments
    let mut segs = Vec::new();

    for version in min_version..=max_version {
        if version == min_version || version == 10 || version == 27 {
            segs = make_segments_optimally_at_version(&code_points, Version::new(version));
        }
        let version = Version::new(version);

        // Check if the segments fit
        let data_capacity_bits = get_num_data_codewords(version, ecc) * 8;
        let data_used_bits = get_total_bits(&segs, version);

        if let Some(data_used_bits) = data_used_bits {
            if data_used_bits <= data_capacity_bits {
                return Some(segs); // This version number is found to be suitable
            }
        }
    }

    None
}

// Returns a new list of segments that is optimal for the given text at the given version number.
fn make_segments_optimally_at_version(code_points: &[char], version: Version) -> Vec<QrSegment> {
    let char_modes = compute_character_modes(code_points, version);
    split_into_segments(code_points, &char_modes)
}

// Returns a new array representing the optimal mode per code point based on the given text and version.
fn compute_character_modes(code_points: &[char], version: Version) -> Vec<QrSegmentMode> {
    // Segment header sizes, measured in 1/6 bits
    let mut head_costs = [0usize; NUM_MODES];

    for i in 0..NUM_MODES {
        head_costs[i] = (4 + num_char_count_bits(MODE_TYPES[i], version) as usize) * 6;
    }

    // charModes[i][j] represents the mode to encode the code point at index i
    // such that the final segment ends in modeTypes[j] and the total number of bits is minimized over all possible choices
    let mut char_modes = vec![[None::<QrSegmentMode>; NUM_MODES]; code_points.len()];

    // At the beginning of each iteration of the loop below,
    // prevCosts[j] is the exact minimum number of 1/6 bits needed to encode the entire string prefix of length i, and end in modeTypes[j]
    let mut prev_costs = head_costs;

    // Calculate costs using dynamic programming
    for i in 0..code_points.len() {
        let c = code_points[i];
        let mut cur_costs = [0usize; NUM_MODES];

        {
            // Always extend a byte mode segment
            cur_costs[0] = prev_costs[0] + c.len_utf8() * 8 * 6;
            char_modes[i][0] = Some(MODE_TYPES[0]);
        }

        // Extend a segment if possible
        if ALPHANUMERIC_CHARSET.contains(&c) {
            // Is alphanumeric
            cur_costs[1] = prev_costs[1] + 33; // 5.5 bits per alphanumeric char
            char_modes[i][1] = Some(MODE_TYPES[1]);
        }
        if ('0'..='9').contains(&c) {
            // Is numeric
            cur_costs[2] = prev_costs[2] + 20; // 3.33 bits per digit
            char_modes[i][2] = Some(MODE_TYPES[2]);
        }
        #[cfg(feature = "kanji")]
        {
            if is_kanji(c) {
                cur_costs[3] = prev_costs[3] + 78; // 13 bits per Shift JIS char
                char_modes[i][3] = Some(MODE_TYPES[3]);
            }
        }

        // Start new segment at the end to switch modes
        for j in 0..NUM_MODES {
            // To mode
            for k in 0..NUM_MODES {
                // From mode
                let new_cost = (cur_costs[k] + 5) / 6 * 6 + head_costs[j];
                if char_modes[i][k].is_some()
                    && (char_modes[i][j].is_none() || new_cost < cur_costs[j])
                {
                    cur_costs[j] = new_cost;
                    char_modes[i][j] = Some(MODE_TYPES[k]);
                }
            }
        }

        prev_costs = cur_costs;
    }

    // Find optimal ending mode
    let mut cur_mode = None::<QrSegmentMode>;

    let mut min_cost = 0;

    for i in 0..NUM_MODES {
        if cur_mode.is_none() || prev_costs[i] < min_cost {
            min_cost = prev_costs[i];
            cur_mode = Some(MODE_TYPES[i]);
        }
    }

    let mut cur_mode = cur_mode.unwrap();

    let mut result = Vec::with_capacity(char_modes.len());
    unsafe {
        result.set_len(char_modes.len());
    }

    // Get optimal mode for each code point by tracing backwards
    for i in (0..char_modes.len()).rev() {
        for (j, e) in MODE_TYPES.iter().copied().enumerate() {
            if e == cur_mode {
                cur_mode = char_modes[i][j].unwrap();
                result[i] = cur_mode;
                break;
            }
        }
    }

    result
}

// Returns a new list of segments based on the given text and modes, such that consecutive code points in the same mode are put into the same segment.
fn split_into_segments(code_points: &[char], char_modes: &[QrSegmentMode]) -> Vec<QrSegment> {
    let mut result = Vec::new();

    // Accumulate run of modes
    let mut cur_mode = char_modes[0];

    let mut start = 0;

    let mut i = 0;
    loop {
        i += 1;

        if i < code_points.len() && char_modes[i] == cur_mode {
            continue;
        }

        let s = &code_points[start..i];

        match cur_mode {
            QrSegmentMode::Byte => {
                let s: String = s.iter().collect();
                let v = s.into_bytes();
                result.push(QrSegment::make_bytes(&v));
            }
            QrSegmentMode::Numeric => {
                result.push(QrSegment::make_numeric(s));
            }
            QrSegmentMode::Alphanumeric => {
                result.push(QrSegment::make_alphanumeric(s));
            }
            QrSegmentMode::Kanji => {
                if cfg!(feature = "kanji") {
                    result.push(make_kanji(s));
                }
            }
            _ => unreachable!(),
        }

        if i >= code_points.len() {
            return result;
        }

        cur_mode = char_modes[i];
        start = i;
    }
}

// Calculates and returns the number of bits needed to encode the given
// segments at the given version. The result is None if a segment has too many
// characters to fit its length field, or the total bits exceeds usize::MAX.
fn get_total_bits(segs: &[QrSegment], version: Version) -> Option<usize> {
    let mut result: usize = 0;
    for seg in segs {
        let ccbits = num_char_count_bits(seg.mode(), version);
        if seg.num_chars() >= 1 << ccbits {
            return None; // The segment's length doesn't fit the field's bit width
        }
        result = result.checked_add(4 + usize::from(ccbits) + seg.data().len())?;
    }
    Some(result)
}

// Returns the bit width of the character count field for a segment in this mode
// in a QR Code at the given version number. The result is in the range [0, 16].
fn num_char_count_bits(mode: QrSegmentMode, ver: Version) -> u8 {
    (match mode {
        QrSegmentMode::Numeric => [10, 12, 14],
        QrSegmentMode::Alphanumeric => [9, 11, 13],
        QrSegmentMode::Byte => [8, 16, 16],
        QrSegmentMode::Kanji => [8, 10, 12],
        QrSegmentMode::Eci => [0, 0, 0],
    })[usize::from((ver.value() + 7) / 17)]
}

#[cfg(feature = "kanji")]
/// Returns a segment representing the specified text string encoded in kanji mode.
fn make_kanji(code_points: &[char]) -> QrSegment {
    let mut bb = BitBuffer(Vec::new());

    for &c in code_points {
        let val = UNICODE_TO_QR_KANJI[c as usize];

        if val == -1 {
            panic!("String contains non-kanji-mode characters");
        }

        bb.append_bits(val as u32, 13);
    }

    QrSegment::new(QrSegmentMode::Kanji, code_points.len(), bb.0)
}

#[cfg(not(feature = "kanji"))]
/// Returns a segment representing the specified text string encoded in kanji mode.
fn make_kanji(_code_points: &[char]) -> QrSegment {
    unreachable!()
}

#[cfg(feature = "kanji")]
fn is_kanji(c: char) -> bool {
    let c = c as usize;
    c < UNICODE_TO_QR_KANJI.len() && UNICODE_TO_QR_KANJI[c] != -1
}

// Returns the number of 8-bit data (i.e. not error correction) codewords contained in any
// QR Code of the given version number and error correction level, with remainder bits discarded.
// This stateless pure function could be implemented as a (40*4)-cell lookup table.
fn get_num_data_codewords(ver: Version, ecl: QrCodeEcc) -> usize {
    get_num_raw_data_modules(ver) / 8
        - table_get(&ECC_CODEWORDS_PER_BLOCK, ver, ecl)
            * table_get(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl)
}

// Returns the number of data bits that can be stored in a QR Code of the given version number, after
// all function modules are excluded. This includes remainder bits, so it might not be a multiple of 8.
// The result is in the range [208, 29648]. This could be implemented as a 40-entry lookup table.
fn get_num_raw_data_modules(ver: Version) -> usize {
    let ver = usize::from(ver.value());
    let mut result: usize = (16 * ver + 128) * ver + 64;
    if ver >= 2 {
        let numalign: usize = ver / 7 + 2;
        result -= (25 * numalign - 10) * numalign - 55;
        if ver >= 7 {
            result -= 36;
        }
    }
    assert!((208..=29648).contains(&result));
    result
}

// Returns an entry from the given table based on the given values.
fn table_get(table: &'static [[i8; 41]; 4], ver: Version, ecl: QrCodeEcc) -> usize {
    table[ecl as usize][usize::from(ver.value())] as usize
}
