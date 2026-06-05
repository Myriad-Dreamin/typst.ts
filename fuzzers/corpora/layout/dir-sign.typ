
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(ltr.sign(), 1)
#test(rtl.sign(), -1)
#test(ttb.sign(), 1)
#test(btt.sign(), -1)