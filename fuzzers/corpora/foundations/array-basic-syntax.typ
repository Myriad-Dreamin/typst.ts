
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 150pt)

// Empty.
#()

// Not an array, just a parenthesized expression.
#(1)

// One item and trailing comma.
#(-1,)

// No trailing comma.
#(true, false)

// Multiple lines and items and trailing comma.
#("1"
    , rgb("002")
    ,)