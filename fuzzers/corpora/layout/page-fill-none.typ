
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test disabling page fill.
// The PNG is filled with black anyway due to the test runner.
#set page(fill: none)
#rect(fill: green)