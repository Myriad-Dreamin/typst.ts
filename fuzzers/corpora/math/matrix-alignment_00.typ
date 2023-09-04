
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alternating alignment in a vector.
$ vec(
  "a" & "a a a" & "a a",
  "a a" & "a a" & "a",
  "a a a" & "a" & "a a a",
) $
