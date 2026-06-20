
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(direction.to(left), rtl)
#test(direction.to(right), ltr)
#test(direction.to(top), btt)
#test(direction.to(bottom), ttb)