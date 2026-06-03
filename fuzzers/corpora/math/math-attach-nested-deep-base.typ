
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test attachments when the base has attachments and is nested arbitrarily
// deep.
#{
  let var = $x^1$
  for i in range(24) {
    var = $var$
  }
  $var_2$
}