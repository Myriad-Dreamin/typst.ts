
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(direction.from(left), ltr)
#test(direction.from(right), rtl)
#test(direction.from(top), ttb)
#test(direction.from(bottom), btt)