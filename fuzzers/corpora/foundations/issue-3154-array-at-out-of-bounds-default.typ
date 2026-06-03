
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let array = (1,)
  test(array.at(1, default: 0), 0)
}