
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stretching along horizontal axis.
#let ext(sym) = math.stretch(sym, size: 2em)
$ ext(arrow.r) quad ext(arrow.l.double.bar) \
  ext(harpoon.rb) quad ext(harpoons.ltrb) \
  ext(paren.t) quad ext(shell.b) \
  ext(eq) quad ext(equiv) $