
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test explicit left alignment in a matrix.
$ mat(
  &"a", &"a a a", &"a a";
  &"a a", &"a a", &"a";
  &"a a a", &"a", &"a a a";
) $
