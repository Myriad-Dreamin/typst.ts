
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the type of `sys.version`
#test(type(sys.version), version)