
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import path("modules/chap1.typ") as chap1
#import chap1.module-path as f
#test(f.b, 1)