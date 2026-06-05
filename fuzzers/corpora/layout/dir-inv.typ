
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(ltr.inv(), rtl)
#test(rtl.inv(), ltr)
#test(ttb.inv(), btt)
#test(btt.inv(), ttb)