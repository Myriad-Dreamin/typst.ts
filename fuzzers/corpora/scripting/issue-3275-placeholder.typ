
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Placeholder.
#for _ in (1, 2) {}
#for _ in (a: 1, b: 2) {}
#for _ in "foo" {}
#for _ in bytes("😊") {}