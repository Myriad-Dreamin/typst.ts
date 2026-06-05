
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "module.typ"
#import module.chap2
#test(chap2.name, "Peter")