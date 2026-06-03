
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the unicode function.
#test(str.from-unicode(97), "a")
#test(str.to-unicode("a"), 97)