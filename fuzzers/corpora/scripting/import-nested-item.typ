
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested item imports.
#import "modules/chap1.typ" as orig-chap1
#import "modules/chap2.typ" as orig-chap2
#import "module.typ": chap2, chap2.name, chap2.chap1, chap2.chap1.name as othername
#test(chap2, orig-chap2)
#test(chap1, orig-chap1)
#test(name, "Peter")
#test(othername, "Klaus")