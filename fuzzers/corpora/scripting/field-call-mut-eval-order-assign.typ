
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test evaluation order when assigning to a variable in an argument.
#{
  let what = ()
  what.insert("what", what = (:))
  test(what, (what: none))
}