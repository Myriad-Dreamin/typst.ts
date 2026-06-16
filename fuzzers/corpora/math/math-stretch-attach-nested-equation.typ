// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stretching with attachments when nested in an equation.
#let body = $stretch(=)$
$ body^"text" $

#{
  let body = $stretch(=)$
  for i in range(24) {
    body = $body$
  }
  $body^"long text"$
}