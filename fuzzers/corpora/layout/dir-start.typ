
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(ltr.start(), left)
#test(rtl.start(), right)
#test(ttb.start(), top)
#test(btt.start(), bottom)