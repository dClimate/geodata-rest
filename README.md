# geodata-rest

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [MongoDB](https://docs.mongodb.com/manual/installation/)

### Features

* Role-based jsonwebtoken authentication.
* Layered configuration system, based on [config-rs](https://github.com/mehcode/config-rs)
* Logging, based on [tracing](https://github.com/tokio-rs/tracing)
* Error handling
* [Polymorphic](https://docs.mongodb.com/manual/reference/geojson/#geometrycollection) [geospatial data and queries](https://docs.mongodb.com/manual/geospatial-queries/#geospatial-queries)  on [Mongodb Atlas](https://www.mongodb.com/atlas/database) with Replica Set support
* Data validation based on keccak-256 hashing, anchored on blockchain

### Demo
* see README-demo.md for running demo curl commands

### TODO:
* promote Roles from string array to struct/collection model
* Add integration test layer with sample data (current test is an axum example)
* Consider replacing bcrypt with argon2
* Consider [multi-hash](https://github.com/multiformats/rust-multihash)
* Move validation endpoint to externally scheduled daemon process
* Implement Docker runtimes

### Next steps:
* Design and implement geospatial data schemas, indexes and queries, input process
* Implement external juno test blockchain for anchoring/validation
* Client app integrating above functionality

