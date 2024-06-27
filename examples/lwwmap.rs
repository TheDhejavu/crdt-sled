use crdt_sled::LWWMap;

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
