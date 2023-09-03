
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test forcing scripts and limits.
$ limits(A)_1^2 != A_1^2 $
$ scripts(sum)_1^2 != sum_1^2 $
$ limits(integral)_a^b != integral_a^b $
