
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
_abc $sin(x) "abc"$_ \
$italic(sin(x) "abc" #box[abc])$ \
*abc $sin(x) "abc"$* \
$bold(sin(x) "abc" #box[abc])$ \