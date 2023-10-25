# Anydrop - WIP

## Design Goals

Akka inspired, but different in some very fundamental ways.  
Event Sourced, but not CQRS.  
In-memory computation.  
Highly concurrent, distributed, and fault-tolerant.  
Embraces Component Based Software Engineering (CBSE).  

## Key Components

* scylladb: as distributed event store
* redb: for in-node state store
* axum: for http endpoint

### Why?

for in-node state store, there's two other options: sled and surrealdb (with rocksdb backend). redb is much lighter than surrealdb, and performance is better than sled (probably, and sled seems to be inactive for a while).
