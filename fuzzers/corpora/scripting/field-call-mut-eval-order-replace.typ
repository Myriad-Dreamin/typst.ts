
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether replacing a field while accessing it causes an error.
#{
  let dict = (one: ())
  dict.one.insert("two", dict.insert("one", (:)))
  test(dict.one, (two: none))
}