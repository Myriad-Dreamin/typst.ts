
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test how math styles fallback.
$upright(frak(bold(alpha))) = upright(bold(alpha)) \
bold(mono(ϝ)) = bold(ϝ) \
sans(Theta) = bold(sans(Theta)) \
bold(upright(planck)) != planck \
bb(e) != italic(bb(e)) \
serif(sans(A)) != serif(A)$