
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Here, `best` was accessed as a variable, where it shouldn't have.
#{
  (best: _) = (best: "brr")
}