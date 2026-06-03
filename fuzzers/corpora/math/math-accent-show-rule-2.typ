
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let rhat(x) = {
  show "\u{0302}": set text(red)
  math.hat(x)
}
$hat(x)$, $rhat(x)$, $hat(rhat(x))$, $rhat(hat(x))$, x\u{0302}