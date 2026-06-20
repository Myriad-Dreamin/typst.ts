
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This used to error, but math now correctly ignores the mutable method name.
#import "module.typ"
#let indirect = (mod: module)
#test($indirect.mod.push(#2)$, $#3$)