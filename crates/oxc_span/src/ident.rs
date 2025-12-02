#![expect(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::len_without_is_empty
)] // TODO
use oxc_allocator::{Allocator, CloneIn, Dummy, FromIn};

use rustc_hash::FxHasher;

use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{Atom, CompactStr, ContentEq};

#[derive(Clone, Copy, Eq)]
pub struct Ident<'a> {
    ptr: NonNull<u8>,
    // Length in bottom 32 bits, hash in top 32 bits
    len_and_hash: u64,
    _marker: PhantomData<&'a str>,
}

impl<'a> Ident<'a> {
    pub fn new(s: &str) -> Self {
        let ptr = NonNull::from(s).cast::<u8>();

        // Produce a hash of the string
        // NOTE: This is creating a fresh hasher for each identifier, which is probably bad for performance?
        // But, I want to see how terrible it is and keep the API simple for testing.
        let hash = {
            let mut hasher = FxHasher::default();
            s.hash(&mut hasher);
            hasher.finish()
        };

        // With FxHasher, highest entropy is in top 32 bits. Clear bottom 32 bits.
        let hash = hash & !(u32::MAX as u64);
        // We know `s.len()` is <= u32::MAX so don't bother masking it
        let len = s.len() as u64;

        // Identifiers cannot have zero size.
        assert!(len > 0, "identifiers cannot have zero length");

        let len_and_hash = len | hash;

        Self { ptr, len_and_hash, _marker: PhantomData }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // Length is in bottom 32 bits
        self.len_and_hash as u32 as usize
    }

    pub fn as_str(self) -> &'a str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.as_ptr(), self.len());
            std::str::from_utf8_unchecked(slice)
        }
    }

    #[inline]
    pub fn into_atom(&self) -> Atom<'a> {
        Atom::from(self.as_str())
    }

    #[inline]
    pub fn from_cow_in(value: &Cow<'a, str>, allocator: &'a Allocator) -> Ident<'a> {
        match value {
            Cow::Borrowed(s) => Ident::from(*s),
            Cow::Owned(s) => Ident::from_in(s, allocator),
        }
    }

    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }
}

impl PartialEq for Ident<'_> {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        // Skip full string comparison unless *both* length and hash match.
        // So we get faster `==` as well as faster hashing.
        self.len_and_hash == other.len_and_hash && self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

// impl PartialEq<Atom<'_>> for Ident<'_> {
//     #[inline]
//     fn eq(&self, other: &Atom<'_>) -> bool {
//         self.as_str() == other.as_str()
//     }
// }

// impl PartialEq<&str> for Ident<'_> {
//     #[inline]
//     fn eq(&self, other: &&str) -> bool {
//         self.as_str() == *other
//     }
// }

impl PartialEq<Ident<'_>> for Cow<'_, str> {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl ContentEq for Ident<'_> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl AsRef<str> for &Ident<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Ident<'_> {
    type Target = str;

    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> From<&'a str> for Ident<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> From<Ident<'a>> for Atom<'a> {
    #[inline]
    fn from(val: Ident<'a>) -> Self {
        val.into_atom()
    }
}

impl<'a> From<Atom<'a>> for Ident<'a> {
    #[inline]
    fn from(atom: Atom<'a>) -> Self {
        Self::new(atom.as_str())
    }
}

impl<'a> From<Ident<'a>> for CompactStr {
    #[inline]
    fn from(ident: Ident<'a>) -> Self {
        ident.into_compact_str()
    }
}

impl<'a> From<Ident<'a>> for Cow<'a, str> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from(ident: Ident<'a>) -> Self {
        Cow::Borrowed(ident.as_str())
    }
}

impl<'alloc> FromIn<'alloc, &Ident<'alloc>> for Ident<'alloc> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from_in(s: &Ident<'alloc>, _: &'alloc Allocator) -> Self {
        *s
    }
}

impl<'alloc> FromIn<'alloc, &str> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(&s))
    }
}

impl<'alloc> FromIn<'alloc, &String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

// TODO: Add rest of `FromIn` impls for `Ident`

impl<'new_alloc> CloneIn<'new_alloc> for Ident<'_> {
    type Cloned = Ident<'new_alloc>;

    #[inline]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Ident::from_in(self.as_str(), allocator)
    }
}

impl<'a> Dummy<'a> for Ident<'a> {
    /// Create a dummy [`Ident`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Ident::new("x")
    }
}

// Only use this `Hash` impl with `IdentHashMap`
impl Hash for Ident<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // Hash is stored in top 32 bits of `len_and_hash`.
        // `hashbrown` only uses top 7 bits and bottom N bits of hash,
        // where N is the exponent of number of buckets in the hash table.
        // So just rotate by 25 bits, so 32-bit hash occupies top 7 bits
        // and bottom 25 bits. This gives good entropy as long as HashMap
        // contains less than 58 million entries ((1 << 25) * 7 / 8).
        // The bits in the middle have low entropy, but that doesn't matter
        // because `hashbrown` will ignore them anyway.
        let hash = self.len_and_hash.rotate_left(32 - 7);
        hasher.write_u64(hash);
    }
}

impl std::fmt::Debug for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.as_str(), serializer)
    }
}

#[cfg(feature = "serialize")]
impl ESTree for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        ESTree::serialize(self.as_str(), serializer);
    }
}

#[cfg(test)]
mod test {
    use super::Ident;

    #[test]
    fn ident_eq() {
        let foo = Ident::new("foo");
        let foo2 = Ident::new("foo");
        let bar = Ident::new("bar");
        assert_eq!(foo, foo2);
        assert_ne!(foo, bar);
    }

    #[test]
    fn ident_as_str() {
        let s = "hello_world";
        let ident = Ident::new(s);
        assert_eq!(ident.as_str(), s);
    }

    #[test]
    fn ident_len() {
        let ident = Ident::new("hello");
        assert_eq!(ident.len(), 5);
    }

    // Expect that `Ident::new` will panic on zero-length strings
    #[test]
    #[should_panic(expected = "identifiers cannot have zero length")]
    fn ident_zero_length() {
        let _ = Ident::new("");
    }
}
