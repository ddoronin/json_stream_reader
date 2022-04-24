use std::collections::HashSet;

lazy_static! {
    pub static ref EMPTY_CHAR_SET: HashSet<u8> =
        vec![b' ', b'\n', b'\t', b'\r'].into_iter().collect();
    pub static ref DIGIT_CHAR_SET: HashSet<u8> =
        vec![b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'-', b'e']
            .into_iter()
            .collect();
}
