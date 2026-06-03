
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(repr(none), "none")
#test(repr(auto), "auto")
#test(repr(type(none)), "type(none)")
#test(repr(type(auto)), "type(auto)")