
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Negative dimensions
#rect(width: -1cm, fill: gradient.linear(red, blue))[Reverse left]

#rect(width: 1cm, fill: gradient.linear(red, blue))[Left]

#align(center, rect(width: -1cm, fill: gradient.linear(red, blue))[Reverse center])

#align(center, rect(width: 1cm, fill: gradient.linear(red, blue))[Center])

#align(right, rect(width: -1cm, fill: gradient.linear(red, blue))[Reverse right])

#align(right, rect(width: 1cm, fill: gradient.linear(red, blue))[Right])