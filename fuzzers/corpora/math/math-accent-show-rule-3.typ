
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show math.accent: it => {
  show "\u{0300}": set text(green)
  it
}
$grave(x)$, x\u{0300}