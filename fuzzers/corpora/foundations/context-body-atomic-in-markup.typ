
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that context body is parsed as atomic expression.
#let c = [#context "hello".]
#test(c.children.first().func(), (context none).func())
#test(c.children.last(), [.])