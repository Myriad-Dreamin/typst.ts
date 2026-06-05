
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test nested top and bottom accents.
$hat(accent(L, \u{0330})), accent(circle(p), \u{0323}),
  macron(accent(caron(accent(A, \u{20ED})), \u{0333})) \
  breve(accent(eta, \u{032E})) = accent(breve(eta), \u{032E})$