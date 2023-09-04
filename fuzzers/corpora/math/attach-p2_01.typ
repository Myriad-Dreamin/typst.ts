
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test for no collisions between descenders/ascenders and attachments

$ sup_(x in P_i) quad inf_(x in P_i) $
$ op("fff",limits: #true)^(y) quad op("yyy", limits:#true)_(f) $
