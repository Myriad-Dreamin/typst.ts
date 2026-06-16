
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let scr(it) = text(stylistic-set: 1, $cal(it)$)
$cal(P)_i != scr(P)_i$, $cal(bold(I))_l != bold(scr(I))_l$
$ product.co_(B in scr(B))^(B in scr(bold(B))) cal(B)(X) $