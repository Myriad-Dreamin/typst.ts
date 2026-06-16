
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 15-21 `toml.decode` is deprecated, directly pass bytes to `toml` instead
// Hint: 15-21 it will be removed in Typst 0.15.0
#let _ = toml.decode