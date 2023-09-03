
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Test valid number align values (horizontal)
// #set enum(number-align: start)
// #set enum(number-align: end)
// #set enum(number-align: left)
// #set enum(number-align: right)
// // Error: 25-28 alignment must be horizontal
// #set enum(number-align: top)