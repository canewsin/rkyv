use crate::{
    collections::hash_map::{ArchivedHashMap, HashMapResolver},
    ser::{Allocator, Writer},
    Archive, Deserialize, Serialize,
};
use core::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
};
use hashbrown::HashMap;
use rancor::Fallible;

impl<K: Archive + Hash + Eq, V: Archive, S> Archive for HashMap<K, V, S>
where
    K::Archived: Hash + Eq,
{
    type Archived = ArchivedHashMap<K::Archived, V::Archived>;
    type Resolver = HashMapResolver;

    #[inline]
    unsafe fn resolve(
        &self,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        ArchivedHashMap::resolve_from_len(self.len(), pos, resolver, out);
    }
}

impl<K, V, S, RandomState> Serialize<S> for HashMap<K, V, RandomState>
where
    K: Serialize<S> + Hash + Eq,
    K::Archived: Hash + Eq,
    V: Serialize<S>,
    S: Fallible + Writer + Allocator + ?Sized,
{
    #[inline]
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        unsafe { ArchivedHashMap::serialize_from_iter(self.iter(), serializer) }
    }
}

impl<K, V, D, S> Deserialize<HashMap<K, V, S>, D>
    for ArchivedHashMap<K::Archived, V::Archived>
where
    K: Archive + Hash + Eq,
    K::Archived: Deserialize<K, D> + Hash + Eq,
    V: Archive,
    V::Archived: Deserialize<V, D>,
    D: Fallible + ?Sized,
    S: Default + BuildHasher,
{
    #[inline]
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<HashMap<K, V, S>, D::Error> {
        let mut result =
            HashMap::with_capacity_and_hasher(self.len(), S::default());
        for (k, v) in self.iter() {
            result.insert(
                k.deserialize(deserializer)?,
                v.deserialize(deserializer)?,
            );
        }
        Ok(result)
    }
}

impl<
        K: Hash + Eq + Borrow<AK>,
        V,
        AK: Hash + Eq,
        AV: PartialEq<V>,
        S: BuildHasher,
    > PartialEq<HashMap<K, V, S>> for ArchivedHashMap<AK, AV>
{
    #[inline]
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        if self.len() != other.len() {
            false
        } else {
            self.iter().all(|(key, value)| {
                other.get(key).map_or(false, |v| value.eq(v))
            })
        }
    }
}

impl<K: Hash + Eq + Borrow<AK>, V, AK: Hash + Eq, AV: PartialEq<V>>
    PartialEq<ArchivedHashMap<AK, AV>> for HashMap<K, V>
{
    #[inline]
    fn eq(&self, other: &ArchivedHashMap<AK, AV>) -> bool {
        other.eq(self)
    }
}

// TODO: uncomment
// #[cfg(test)]
// mod tests {
//     use crate::{
//         archived_root,
//         ser::{serializers::AllocSerializer, Serializer},
//         Deserialize,
//     };
//     #[cfg(all(feature = "alloc", not(feature = "std")))]
//     use alloc::string::String;
//     use hashbrown::HashMap;
//     use rancor::Failure;

//     #[test]
//     fn index_map() {
//         let mut value = HashMap::new();
//         value.insert(String::from("foo"), 10);
//         value.insert(String::from("bar"), 20);
//         value.insert(String::from("baz"), 40);
//         value.insert(String::from("bat"), 80);

//         let mut serializer = AllocSerializer::<4096>::default();
//         Serializer::<Failure>::serialize_value(&mut serializer, &value).unwrap();
//         let result = serializer.into_serializer().into_inner();
//         let archived =
//             unsafe { archived_root::<HashMap<String, i32>>(result.as_ref()) };

//         assert_eq!(value.len(), archived.len());
//         for (k, v) in value.iter() {
//             let (ak, av) = archived.get_key_value(k.as_str()).unwrap();
//             assert_eq!(k, ak);
//             assert_eq!(v, av);
//         }

//         let deserialized = Deserialize::<HashMap<String, i32>, _, Failure>::deserialize(archived, &mut ()).unwrap();
//         assert_eq!(value, deserialized);
//     }

//     TODO: uncomment
//     #[cfg(feature = "bytecheck")]
//     #[test]
//     fn validate_index_map() {
//         use crate::check_archived_root;

//         let mut value = HashMap::new();
//         value.insert(String::from("foo"), 10);
//         value.insert(String::from("bar"), 20);
//         value.insert(String::from("baz"), 40);
//         value.insert(String::from("bat"), 80);

//         let mut serializer = AllocSerializer::<4096>::default();
//         Serializer::<Failure>::serialize_value(&mut serializer, &value).unwrap();
//         let result = serializer.into_serializer().into_inner();
//         check_archived_root::<HashMap<String, i32>, Failure>(result.as_ref())
//             .expect("failed to validate archived index map");
//     }
// }
