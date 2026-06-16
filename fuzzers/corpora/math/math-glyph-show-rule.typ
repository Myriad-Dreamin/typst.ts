// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show "+": set text(orange, font: "Noto Sans Math")
$ 1 + 1 = +2 $
#show "+": text(2em)[#sym.plus.o]
$ 1 + 1 = +2 $