// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test #454 equations.
$ mat(-1, 1, 1; 1, -1, 1; 1, 1, -1) $
$ mat(-1&, 1&, 1&; 1&, -1&, 1&; 1&, 1&, -1&) $
$ mat(-1&, 1&, 1&; 1, -1, 1; 1, 1, -1) $
$ mat(&-1, &1, &1; 1, -1, 1; 1, 1, -1) $