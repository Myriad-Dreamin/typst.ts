
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#context test(block.above, auto)
#set block(spacing: 20pt)
#context test(block.above, 20pt)
#context test(block.below, 20pt)