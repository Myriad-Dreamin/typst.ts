
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test insert.
#{
  let dict = (a: 1, b: 2)
  dict.insert("b", 3)
  test(dict, (a: 1, b: 3))
  dict.insert("c", 5)
  test(dict, (a: 1, b: 3, c: 5))
}