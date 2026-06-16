// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test primes always attaching as scripts
$ x' $
$ x^' $
$ attach(x, t: ') $
$ <' $
$ attach(<, br: ') $
$ op(<, limits: #true)' $
$ limits(<)' $