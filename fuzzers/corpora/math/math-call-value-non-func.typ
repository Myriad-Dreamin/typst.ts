
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test($sin(1)$, $sin(#1)$)