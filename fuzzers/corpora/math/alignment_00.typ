
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alignment step functions.
#set page(width: 225pt)
$
"a" &= c \
&= c + 1 & "By definition" \
&= d + 100 + 1000 \
&= x && "Even longer" \
$
