
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 14-20 `xml.decode` is deprecated, directly pass bytes to `xml` instead
// Hint: 14-20 it will be removed in Typst 0.15.0
#let _ = xml.decode