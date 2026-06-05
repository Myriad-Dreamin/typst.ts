
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(ltr.end(), right)
#test(rtl.end(), left)
#test(ttb.end(), bottom)
#test(btt.end(), top)