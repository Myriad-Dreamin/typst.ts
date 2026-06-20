
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let p = "module.typ"
#import p as f
#test(f.b, 1)