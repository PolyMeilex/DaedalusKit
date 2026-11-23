use zstring::{ZString, windows_12xx};

pub fn script_string(s: &str) -> ZString {
    windows_12xx::encode(s)
}

pub fn symbol_name(identifier: &str) -> ZString {
    let mut name = script_string(identifier);
    name.make_ascii_uppercase();
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let original = b"Stra\xBFnik";
        let decoded = windows_12xx::decode(original);
        assert_eq!(decoded, "Strażnik");
        assert_eq!(script_string(&decoded).as_slice(), original);
        assert_eq!(symbol_name(&decoded).as_slice(), b"STRA\xBFNIK");
    }
}
