
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test forcefully attaching primes as limits
$ attach(<, t: ') $
$ <^' $
$ attach(<, b: ') $
$ <_' $

$ limits(x)^' $
$ attach(limits(x), t: ') $