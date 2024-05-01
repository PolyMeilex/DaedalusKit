use rustc_hash::FxHashMap as HashMap;
use std::hash::Hash;
use unicase::UniCase;

enum CaseMap<K, V> {
    Insensitive(HashMap<UniCase<K>, V>),
    Sensitive(HashMap<K, V>),
}

impl<K, V> CaseMap<K, V>
where
    K: AsRef<str> + Eq + Hash,
{
    fn new(case_sensitive: bool) -> Self {
        if case_sensitive {
            Self::Sensitive(HashMap::default())
        } else {
            Self::Insensitive(HashMap::default())
        }
    }

    fn is_case_sensitive(&self) -> bool {
        matches!(self, Self::Sensitive(_))
    }

    fn get(&self, key: K) -> Option<&V> {
        match self {
            Self::Insensitive(map) => map.get(&UniCase::new(key)),
            Self::Sensitive(map) => map.get(&key),
        }
    }

    fn insert(&mut self, key: K, v: V) {
        match self {
            Self::Insensitive(map) => map.insert(UniCase::new(key), v),
            Self::Sensitive(map) => map.insert(key, v),
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Interned(u32);

pub struct Interner {
    map: CaseMap<StrPtr, Interned>,
    list: Vec<String>,
}

impl Interner {
    pub fn new(case_sensitive: bool) -> Self {
        Self {
            map: CaseMap::new(case_sensitive),
            list: Vec::new(),
        }
    }

    pub fn intern(&mut self, v: &str) -> Interned {
        if let Some(id) = self.map.get(unsafe { StrPtr::new(v) }) {
            return *id;
        }

        // TODO: We could store multiple strings per allocation
        let buff = if self.map.is_case_sensitive() {
            v.to_string()
        } else {
            v.to_uppercase()
        };

        let idx = Interned(self.list.len() as u32);

        // SAFETY: buff is kept alive and never mutated
        let key = unsafe { StrPtr::new(buff.as_str()) };
        self.map.insert(key, idx);

        self.list.push(buff);

        idx
    }

    pub fn get(&self, id: Interned) -> &str {
        self.list[id.0 as usize].as_str()
    }
}

struct StrPtr(*const str);

impl StrPtr {
    // SAFETY:
    // - Source strings are never mutated
    // - Source strings are dropped only after all UnsafeStr are dropped
    unsafe fn new(v: &str) -> Self {
        Self(v)
    }
}

impl AsRef<str> for StrPtr {
    fn as_ref(&self) -> &str {
        // SAFETY:
        // - We never expose this type
        // - Source strings are never mutated
        // - Source strings are dropped only after all UnsafeStr are dropped
        unsafe { &*self.0 }
    }
}

impl Hash for StrPtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl Eq for StrPtr {}
impl PartialEq for StrPtr {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern() {
        let mut interner = Interner::new(false);

        let id1 = interner.intern("abc");
        assert_eq!(interner.get(id1), "ABC");
        let id2 = interner.intern("AbC");
        assert_eq!(interner.get(id2), "ABC");
    }
}
