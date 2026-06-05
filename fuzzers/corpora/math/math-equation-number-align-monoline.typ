
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set math.equation(numbering: "(1)")
$ p = sum_k k ln a $

#set math.equation(numbering: "(1)", number-align: top)
$ p = sum_k k ln a $

#set math.equation(numbering: "(1)", number-align: bottom)
$ p = sum_k k ln a $