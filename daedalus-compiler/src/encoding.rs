use zstring::ZString;

pub fn script_string(s: &str) -> ZString {
    let (bytes, _, _) = encoding_rs::WINDOWS_1250.encode(s);
    ZString::from(bytes.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let original = b"Stra\xBFnik";
        let (decoded, _, _) = encoding_rs::WINDOWS_1250.decode(original);
        assert_eq!(decoded, "Strażnik");
        assert_eq!(script_string(&decoded).as_slice(), original);
    }
}
