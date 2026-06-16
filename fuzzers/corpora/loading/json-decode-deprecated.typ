
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 15-21 `json.decode` is deprecated, directly pass bytes to `json` instead
// Hint: 15-21 it will be removed in Typst 0.15.0
#let _ = json.decode