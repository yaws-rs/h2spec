# h2spec Blueprint

Implements the [yaws](https://yaws-rs.github.io/book) Orbit and BluePrint for [H2spec](https://yaws-rs.github.io/book/protocols/h2spec.html) usage.

## Usage
```rust,ignore
use blueprint::{BluePrint, Left, Orbit};
use blueprint_h2spec::{H2Serving, H2SpecServer};
use yaws_app_rick_roll::RickRoll;

use yaws_app_rick_roll::RickRoll;

// Rick Roll is a HTTP/1.1 App but we wrap it into H2Serving
fn init_orbit() -> H2Serving<MyH2App<RickRoll>> {
    H2SpecServer::with_defaults().unwrap()
}
```

See [an example](https://github.com/yaws-rs/h2spec/tree/main/examples/basic-h2-cleartext).
