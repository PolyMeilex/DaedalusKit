use std::{
    borrow::Borrow,
    io::{Read, Write},
};

use bstr::{BStr, BString};
use byteorder::{ReadBytesExt as _, WriteBytesExt as _};

pub use bstr;
pub use bstr::{ByteSlice, ByteVec};

#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl std::ops::Deref for ZString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ZString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<[u8]> for ZString {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Borrow<BStr> for ZString {
    #[inline]
    fn borrow(&self) -> &BStr {
        self.0.as_bstr()
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

impl AsRef<[u8]> for ZString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<BStr> for ZString {
    #[inline]
    fn as_ref(&self) -> &BStr {
        self.0.as_bstr()
    }
}

impl ZString {
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
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
