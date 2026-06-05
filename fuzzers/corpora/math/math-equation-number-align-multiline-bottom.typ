
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show math.equation: set align(left)
#set math.equation(numbering: "(1)", number-align: bottom)

$ p &= ln a b \
    &= ln a + ln b $
$ q &= sum_k ln A \
    &= sum_k k ln a $