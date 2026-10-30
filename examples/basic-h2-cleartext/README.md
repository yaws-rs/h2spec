# H2spec Example

This example embeds regular YAWS HTTP/1.1 applications into H2 Serving context.

We have set it to embed RickRoll H11 Application that spits him dancing out every chance it gets.

## Run

Download your chosen Rick Roll gif into /tmp/rick-roll.gif

Run the server:
```ignore
env RUST_LOG=info cargo run
```

Point curl at it:
```ignore
curl --http2-prior-knowledge http://localhost:7878 --output /tmp/test-rick.gif
```

Feel free to ensure that Rick was pure:
```ignore
sha256sum /tmp/rick-roll.gif
sha256sum /tmp/test-rick.gif
```
