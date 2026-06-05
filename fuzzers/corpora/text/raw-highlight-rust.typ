
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)

```rust
/// A state machine.
#[derive(Debug)]
enum State<'a> { A(u8), B(&'a str) }

fn advance(state: State<'_>) -> State<'_> {
    unimplemented!("state machine")
}
```