## Tests

Run integration test:

```
cargo insta test --manifest-path ./integration/Cargo.toml --no-force-pass -- --nocapture
```

Review integration test:

```
cargo insta review --manifest-path ./integration/Cargo.toml
```
