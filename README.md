# geodata-rest

### Requirements

* [Rust](https://www.rust-lang.org/tools/install)
* [MongoDB](https://docs.mongodb.com/manual/installation/)
* [Docker](https://www.docker.com/) (for integration test)
### development

* run tests:
```sh
cargo test
```
### Features

* Role-based jsonwebtoken authentication.
* Layered configuration system, based on [config-rs](https://github.com/mehcode/config-rs)
* Logging, based on [tracing](https://github.com/tokio-rs/tracing)
* Error handling
* [Polymorphic](https://docs.mongodb.com/manual/reference/geojson/#geometrycollection) [geospatial data and queries](https://docs.mongodb.com/manual/geospatial-queries/#geospatial-queries)  on [Mongodb Atlas](https://www.mongodb.com/atlas/database) with Replica Set support
* Data validation based on keccak-256 hashing, anchored on blockchain
* Contract messaging via [cosmrs](https://github.com/cosmos/cosmos-rust)
* Full integration test coverage

### Diagram

![diagram](./assets/diagram.png)

### Dev
* For [contract](https://github.com/dclimate/geodata-anchor) changes, copy msg.rs and geodata_anchor.wasm from contract to common directory, e.g.: 'cp ../geodata-anchor/src/msg.rs common' and 'cp ../geodata-anchor/artifacts/geodata_anchor.wasm assets'
* With a connected testnet, the deployed contract address would be supplied via config/default.json. Currently the integration test is running juno via docker and supplies the contract address via env variable.
* To supply private credentials for Mongodb Atlas cluster overriding the default localhost instance, add config/local.json (included in .gitignore). Sample:
  {
    "database": {
      "uri": "mongodb+srv://xxxxxxcluster0.xxxx.mongodb.net",
      "name": "geomancy"
    },
  
    "auth": {
      "secret": "xxxxxx"
    }
  }
* see README-dev.md for running dev curl commands

### Next steps:
* Consider replacing bcrypt with argon2
* Consider [multi-hash](https://github.com/multiformats/rust-multihash)
* Move validation endpoint to externally scheduled daemon process
* Implement Docker runtimes
* Design and implement geospatial data schemas, indexes and queries, input process
* Implement external juno test blockchain for anchoring/validation
* Client app integrating above functionality

