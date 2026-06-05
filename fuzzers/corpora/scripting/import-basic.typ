
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test basic syntax and semantics.

// Test that this will be overwritten.
#let value = [foo]

// Import multiple things.
#import "module.typ": fn, value
#fn[Like and Subscribe!]
#value

// Should output `bye`.
// Stop at semicolon.
#import "module.typ": a, c;bye