
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


// Test transition from script to scriptscript.
#[
#set text(size:20pt)
$  e^(e^(e^(e))) $
]
A large number: $e^(e^(e^(e)))$.
