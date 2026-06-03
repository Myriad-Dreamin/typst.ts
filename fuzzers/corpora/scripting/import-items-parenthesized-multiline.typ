
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "module.typ": (
  a
)
#import "module.typ": (
  a, b as e,
  c,


      d,
)

#test(a, none)
#test(e, 1)
#test(c, 2)
#test(d, 3)