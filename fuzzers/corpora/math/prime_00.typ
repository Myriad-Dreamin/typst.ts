
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test prime symbols after code mode.
#let g = $f$
#let gg = $f$

$
  #(g)' #g' #g ' \
  #g''''''''''''''''' \
  gg'
$
