
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 2-45 floating placement must be `auto`, `top`, or `bottom`
// #place(center + horizon, float: true)[Hello]