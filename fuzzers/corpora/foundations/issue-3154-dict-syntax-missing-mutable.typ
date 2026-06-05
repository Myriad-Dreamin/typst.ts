
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let dict = (a: 1)
  dict.b = 9
  test(dict, (a: 1, b: 9))
}