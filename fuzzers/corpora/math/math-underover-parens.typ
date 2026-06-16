
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test parentheses.
$ overparen(
  underparen(x + y, "long comment"),
  1 + 2 + ... + 5  
) $