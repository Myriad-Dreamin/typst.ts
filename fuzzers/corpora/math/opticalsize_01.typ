
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

//  Test prime/double prime via scriptsize
#let prime = [ \u{2032} ]
#let dprime = [ \u{2033} ]
#let tprime = [ \u{2034} ]
$ y^dprime-2y^prime + y = 0 $
$y^dprime-2y^prime + y = 0$
$ y^tprime_3 + g^(prime 2) $
