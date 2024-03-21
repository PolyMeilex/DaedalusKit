use std::io::{Read, Write};

use bstr::{BStr, BString, ByteSlice};
use byteorder::{ReadBytesExt as _, WriteBytesExt as _};

#[derive(Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZString(pub BString);

impl std::fmt::Display for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.as_bstr().iter() {
            if *b == 0xFF {
                write!(f, "$")?;
            } else {
                write!(f, "{}", *b as char)?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for ZString {
    #[inline]
    fn from(s: &'a [u8; N]) -> Self {
        Self(BString::from(&s[..]))
    }
}

impl<const N: usize> From<[u8; N]> for ZString {
    #[inline]
    fn from(s: [u8; N]) -> Self {
        Self::from(&s[..])
    }
}

impl<'a> From<&'a [u8]> for ZString {
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        Self::from(s.to_vec())
    }
}

impl From<Vec<u8>> for ZString {
    #[inline]
    fn from(s: Vec<u8>) -> Self {
        Self(BString::new(s))
    }
}

impl<'a> From<&'a str> for ZString {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::from(s.as_bytes().to_vec())
    }
}

impl ZString {
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn encode(&self, mut w: impl Write) -> std::io::Result<usize> {
        let len = self.len();
        w.write_all(self.as_bytes())?;
        w.write_u8(b'\n')?;
        Ok(len + 1)
    }

    pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
        let mut str = Vec::new();
        let mut b = r.read_u8()?;
        while b != b'\n' {
            // if b == 0xFF {
            //     Set some "builin" flag
            // }
            str.push(b);
            b = r.read_u8()?;
        }

        Ok(Self(BString::new(str)))
    }
}
