
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether replacing a nested field while accessing it causes an error.
#{
  let dict = (one: (two: ()))
  dict.one.two.insert("three", dict.insert("one", (two: (:))))
  test(dict.one, (two: (three: none)))
}