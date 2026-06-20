
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test valid marker align values (horizontal and vertical)
#set enum(number-align: start)
#set enum(number-align: end)
#set enum(number-align: left)
#set enum(number-align: center)
#set enum(number-align: right)
#set enum(number-align: top)
#set enum(number-align: horizon)
#set enum(number-align: bottom)
#set enum(number-align: start + top)
#set enum(number-align: left + horizon)
#set enum(number-align: center + bottom)