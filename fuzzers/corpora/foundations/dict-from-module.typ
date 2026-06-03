
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test dictionary constructor
#test(type(dictionary(sys).at("version")), version)
#test(dictionary(sys).at("no-crash", default: none), none)