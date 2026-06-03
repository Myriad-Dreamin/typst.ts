
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `counter`.
#let c = counter("heading")
#c.update(2)
#c.update(n => n + 2)
#context test(c.get(), (4,))
#c.update(n => n - 3)
#context test(c.at(here()), (1,))