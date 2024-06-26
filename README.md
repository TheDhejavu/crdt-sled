# crdt-sled
Convergent and Commutative Replicated Data Types (CRDTs) for Sled.

## Introduction
This repository provides an implementation of Convergent and Commutative Replicated Data Types (CRDTs) using the Sled embedded database. CRDTs are data structures that provide strong eventual consistency and are designed to operate in distributed systems. They allow multiple replicas to be updated independently and concurrently without coordination, and can be merged automatically to form a consistent state.

## Available CRDTs
Currently, this repository includes the implementation of the following CRDT:

### Last-Write-Wins Map (LWWMap)
The Last-Write-Wins Map (LWWMap) is a type of CRDT that resolves conflicts based on timestamps. In an LWWMap, each entry is associated with a timestamp, and when conflicts occur (e.g., two entries with the same key but different values), the entry with the most recent timestamp is chosen.

### Future Work

- **G-Counter**: A grow-only counter that supports only increment operations.
- **PN-Counter**: A counter that supports both increment and decrement operations.
- **OR-Set**: An observed-remove set that allows elements to be added and removed, handling concurrent operations gracefully.
- **LWW-Register**: A register that holds a single value with the most recent timestamp winning on conflicts.

## References
- [A comprehensive study of Convergent and Commutative Replicated Data Types](https://inria.hal.science/file/index/docid/555588/filename/techreport.pdf)
- [Rust CRDT](https://github.com/rust-crdt/rust-crdt)