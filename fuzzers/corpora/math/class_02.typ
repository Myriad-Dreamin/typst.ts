
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test nested.
#let normal = math.class.with("normal")
#let pluseq = $class("binary", normal(+) normal(=))$
$ a pluseq 5 $
