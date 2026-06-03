
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test values greater than 32-bits
#let c = counter("c")
#c.update(100000000001)
#context test(c.get(), (100000000001,))
#c.step()
#context test(c.get(), (100000000002,))
#c.update(n => n + 2)
#context test(c.get(), (100000000004,))