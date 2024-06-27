use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sled::Db;
use serde_with::{serde_as, DisplayFromStr};
use crate::error::LWWMapError;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LWWElement<T> {
    value: T,
    #[serde_as(as = "DisplayFromStr")]
    timestamp: DateTime<Utc>,
}

impl<T> LWWElement<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct LWWMap<K, V> {
    db: Db,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> LWWMap<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + std::hash::Hash + Eq + Clone ,
    V: Serialize + for<'de> Deserialize<'de> + Clone + std::fmt::Debug,
{
    pub fn new(db: Db) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn insert(&self, key: K, value: V) -> Result<(), LWWMapError> {
        let element = LWWElement::new(value);
        let key_bytes = serde_json::to_vec(&key)?;
        let value_bytes = serde_json::to_vec(&element)?;
        self.db.insert(key_bytes, value_bytes)?;
        Ok(())
    }

    pub fn remove(&self, key: &K) -> Result<(), LWWMapError> {
        if let Some(mut element) = self.get_element(key)? {
            element.timestamp = Utc::now(); 
            let key_bytes = serde_json::to_vec(key)?;
            let value_bytes = serde_json::to_vec(&element)?;
            self.db.insert(key_bytes, value_bytes)?;
        }
        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, LWWMapError> {
        if let Some(element) = self.get_element(key)? {
            Ok(Some(element.value))
        } else {
            Ok(None)
        }
    }

    pub fn get_element(&self, key: &K) -> Result<Option<LWWElement<V>>, LWWMapError> {
        let key_bytes = serde_json::to_vec(key)?;
        if let Some(value_bytes) = self.db.get(key_bytes)? {
            let element: LWWElement<V> = serde_json::from_slice(&value_bytes)?;
            Ok(Some(element))
        } else {
            Ok(None)
        }
    }

    pub fn merge(&self, other: &LWWMap<K, V>) -> Result<(), LWWMapError> {
        for entry in other.db.iter() {
            let (key_bytes, value_bytes) = entry?;
            let key: K = serde_json::from_slice(&key_bytes)?;
            let other_element: LWWElement<V> = serde_json::from_slice(&value_bytes)?;

            if let Some(mut element) = self.get_element(&key)? {
                if element.timestamp < other_element.timestamp {
                    element = other_element;
                    self.db.insert(key_bytes, serde_json::to_vec(&element)?)?;
                }
            } else {
                self.db.insert(key_bytes, value_bytes)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, LWWMapError> {
        let mut map = HashMap::new();
        for entry in self.db.iter() {
            let (key_bytes, value_bytes) = entry?;
            let key: K = serde_json::from_slice(&key_bytes)?;
            let element: LWWElement<V> = serde_json::from_slice(&value_bytes)?;
            map.insert(key, element);
        }
        Ok(serde_json::to_string(&map)?)
    }
}