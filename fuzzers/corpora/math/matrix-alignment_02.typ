
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alignment in a matrix.
$ mat(
  "a", "a a a", "a a";
  "a a", "a a", "a";
  "a a a", "a", "a a a";
) $
