
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let dict = (a: 1)
  test(dict.at("b", default: 0), 0)
}