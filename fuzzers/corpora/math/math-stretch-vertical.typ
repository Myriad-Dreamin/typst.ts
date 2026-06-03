
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stretching along vertical axis.
#let ext(sym) = math.stretch(sym, size: 2em)
$ ext(bar.v) quad ext(bar.v.double) quad
  ext(chevron.l) quad ext(chevron.r) quad
  ext(paren.l) quad ext(paren.r) \
  ext(bracket.l.stroked) quad ext(bracket.r.stroked) quad
  ext(brace.l) quad ext(brace.r) quad
  ext(bracket.l) quad ext(bracket.r) $