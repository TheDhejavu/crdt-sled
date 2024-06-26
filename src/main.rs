use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sled::Db;
use serde_with::{serde_as, DisplayFromStr};
use error::LWWMapError;

mod error;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LWWElement<T> {
    value: T,
    #[serde_as(as = "DisplayFromStr")]
    timestamp: DateTime<Utc>,
}

impl<T> LWWElement<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug)]
struct LWWMap<K, V> {
    db: Db,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> LWWMap<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + std::hash::Hash + Eq + Clone ,
    V: Serialize + for<'de> Deserialize<'de> + Clone + std::fmt::Debug,
{
    fn new(db: Db) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }

    fn insert(&self, key: K, value: V) -> Result<(), LWWMapError> {
        let element = LWWElement::new(value);
        let key_bytes = serde_json::to_vec(&key)?;
        let value_bytes = serde_json::to_vec(&element)?;
        self.db.insert(key_bytes, value_bytes)?;
        Ok(())
    }

    fn remove(&self, key: &K) -> Result<(), LWWMapError> {
        if let Some(mut element) = self.get_element(key)? {
            element.timestamp = Utc::now(); // Update timestamp to mark removal
            let key_bytes = serde_json::to_vec(key)?;
            let value_bytes = serde_json::to_vec(&element)?;
            self.db.insert(key_bytes, value_bytes)?;
        }
        Ok(())
    }

    fn get(&self, key: &K) -> Result<Option<V>, LWWMapError> {
        if let Some(element) = self.get_element(key)? {
            Ok(Some(element.value))
        } else {
            Ok(None)
        }
    }

    fn get_element(&self, key: &K) -> Result<Option<LWWElement<V>>, LWWMapError> {
        let key_bytes = serde_json::to_vec(key)?;
        if let Some(value_bytes) = self.db.get(key_bytes)? {
            let element: LWWElement<V> = serde_json::from_slice(&value_bytes)?;
            Ok(Some(element))
        } else {
            Ok(None)
        }
    }

    fn merge(&self, other: &LWWMap<K, V>) -> Result<(), LWWMapError> {
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

    fn to_json(&self) -> Result<String, LWWMapError> {
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

#[tokio::main]
async fn main() {
    let db1 = sled::open("lwwmap1.db").unwrap();
    let map1 = LWWMap::new(db1);

    let db2 = sled::open("lwwmap2.db").unwrap();
    let map2 = LWWMap::new(db2);

    // Simulate Process 1
    map1.insert("key1".to_string(), "value1_process1".to_string()).unwrap();
    map1.insert("key2".to_string(), "value2_process1".to_string()).unwrap();
    map1.remove(&"key2".to_string()).unwrap();

    // Simulate Process 2
    map2.insert("key1".to_string(), "value1_process2".to_string()).unwrap();
    map2.insert("key3".to_string(), "value3_process2".to_string()).unwrap();

    
    println!("Map1 before merge: {}", map1.to_json().unwrap());
    println!("");
    println!("Map2 before merge: {}", map2.to_json().unwrap());
    println!("");
    println!("");

    // Merge map2 into map1
    map1.merge(&map2).unwrap();

    
    println!("Map1 after merge: {}", map1.to_json().unwrap());
    println!("");
    println!("Map2 after merge: {}", map2.to_json().unwrap());
    println!("");
    println!("");

    // Cleanup
    sled::open("lwwmap1.db").unwrap().clear().unwrap();
    sled::open("lwwmap2.db").unwrap().clear().unwrap();
}
