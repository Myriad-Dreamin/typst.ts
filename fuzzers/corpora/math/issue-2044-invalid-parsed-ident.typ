
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, the dot at the end was causing the right parenthesis to be
// parsed as an identifier instead of the closing right parenthesis.
$floor(phi.alt.)$
$floor(phi.alt. )$