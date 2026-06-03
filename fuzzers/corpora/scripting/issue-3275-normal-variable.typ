
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Normal variable.
#for x in (1, 2) {}
#for x in (a: 1, b: 2) {}
#for x in "foo" {}
#for x in bytes("😊") {}