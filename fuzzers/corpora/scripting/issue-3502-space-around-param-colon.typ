
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a space after a named parameter is permissible.
#let f( param : v ) = param
#test(f( param /* ok */ : 2 ), 2)