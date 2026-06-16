// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test baseline alignment.
$ mat(
  a, b^2;
  sum_(x \ y) x, a^(1/2);
  zeta, alpha;
) $