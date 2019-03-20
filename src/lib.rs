// use crate::general::Portlike;
use hashbrown::HashMap;
use std::any::TypeId;
use std::{iter, hash::Hash};
use std::mem::{size_of, transmute};


// TODO consider fragmentation

#[derive(Debug, Copy, Clone)]
pub enum EntryError {
    TypeMismatch,
    ValueRemoved,
    UnknownKey,
}

#[derive(Debug)]
pub struct AnySizedMap<K> where K: Sized + Copy + Eq + Hash {
    allocated: HashMap<K, (usize, TypeId)>, //location, typeid
    bytes: Vec<usize>,
}
impl<K> Default for AnySizedMap<K> where K: Sized + Copy + Eq + Hash {
    fn default() -> Self {
        Self {
            allocated: HashMap::default(),
            bytes: Vec::default(),
        }
    }
}
impl<K> AnySizedMap<K> where K: Sized + Copy + Eq + Hash {
    pub fn insert<T>(&mut self, key: K, datum: T)
    where
        T: Sized + 'static,
    {
        let bytes_needed = size_of::<Option<T>>();
        let tid: TypeId = TypeId::of::<T>();
        let AnySizedMap { allocated, bytes } = self;
        let alloc_key = allocated.entry(key).or_insert_with(|| {
            let alloc_key = (bytes.len(), tid);
            bytes.extend(iter::repeat(0).take(bytes_needed));
            alloc_key
        });

        assert_eq!(tid, alloc_key.1);

        let entry: &mut Option<T> = unsafe { transmute(&mut self.bytes[alloc_key.0]) };
        entry.replace(datum); // TODO prohibit drop?
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.allocated.contains_key(&key)
    }

    pub fn get<T>(&self, key: K) -> Result<&T, EntryError>
    where
        T: Sized + 'static,
    {
        if let Some(alloc_key) = self.allocated.get(&key) {
            let tid: TypeId = TypeId::of::<T>();
            if tid != alloc_key.1 {
                return Err(EntryError::TypeMismatch);
            }
            let entry: &Option<T> = unsafe { transmute(&self.bytes[alloc_key.0]) };
            match entry {
                Some(x) => Ok(x),
                None => Err(EntryError::ValueRemoved),
            }
        } else {
            Err(EntryError::UnknownKey)
        }
    }

    pub fn get_mut<T>(&mut self, key: K) -> Result<&mut T, EntryError>
    where
        T: Sized + 'static,
    {
        if let Some(alloc_key) = self.allocated.get(&key) {
            let tid: TypeId = TypeId::of::<T>();
            if tid != alloc_key.1 {
                return Err(EntryError::TypeMismatch);
            }
            let entry: &mut Option<T> = unsafe { transmute(&mut self.bytes[alloc_key.0]) };
            match entry {
                Some(x) => Ok(x),
                None => Err(EntryError::ValueRemoved),
            }
        } else {
            Err(EntryError::UnknownKey)
        }
    }

    pub fn remove<T>(&mut self, key: K) -> Result<T, EntryError>
    where
        T: Sized + 'static,
    {
        if let Some(alloc_key) = self.allocated.get(&key) {
            let tid: TypeId = TypeId::of::<T>();
            if tid != alloc_key.1 {
                return Err(EntryError::TypeMismatch);
            }
            let entry: &mut Option<T> = unsafe { transmute(&mut self.bytes[alloc_key.0]) };
            match entry.take() {
                Some(x) => Ok(x),
                None => Err(EntryError::ValueRemoved),
            }
        } else {
            Err(EntryError::UnknownKey)
        }
    }
}
