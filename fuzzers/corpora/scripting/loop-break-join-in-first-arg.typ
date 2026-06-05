
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test second block during break flow.

#for i in range(10) {
  table(
    { [A]; break },
    for _ in range(3) [B]
  )
}