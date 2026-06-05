
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Code mode
#{
  import "module.typ": b
  test(b, 1)
}