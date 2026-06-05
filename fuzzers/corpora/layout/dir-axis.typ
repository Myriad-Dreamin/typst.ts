
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(ltr.axis(), "horizontal")
#test(rtl.axis(), "horizontal")
#test(ttb.axis(), "vertical")
#test(btt.axis(), "vertical")