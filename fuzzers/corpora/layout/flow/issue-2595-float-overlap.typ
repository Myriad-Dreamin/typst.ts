
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 80pt)

1
#place(auto, float: true, block(height: 100%, width: 100%, fill: aqua))
#place(auto, float: true, block(height: 100%, width: 100%, fill: red))
#lines(7)