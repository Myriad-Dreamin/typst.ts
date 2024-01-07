
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test prime superscript on large symbol
$ scripts(sum_(k in NN))^prime 1/k^2 $
$sum_(k in NN)^prime 1/k^2$
