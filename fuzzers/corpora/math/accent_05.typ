
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test effect of accent on superscript.
$A^x != hat(A)^x != hat(hat(A))^x$
