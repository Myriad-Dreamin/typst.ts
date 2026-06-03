
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test ignoring leading and trailing ignorant fragments.
#box($ (1 / 2) $)
#box({
  show "(": it => context it
  $ (1 / 2) $
})
#box({
  show ")": it => context it
  $ (1 / 2) $
})
#box({
  show "(": it => context it
  show ")": it => context it
  $ (1 / 2) $
})