
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show math.equation: set text(
  font: (
    // Ignore that this regex actually misses some of the script glyphs...
    (name: "XITS Math", covers: regex("[\u{1D49C}-\u{1D503}]")),
    "New Computer Modern Math"
  ),
  stylistic-set: 1,
)
$ cal(P)_i (X) * cal(C)_1 $