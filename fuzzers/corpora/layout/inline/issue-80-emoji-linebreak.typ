
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that there are no linebreaks in composite emoji (issue #80).
#set page(width: 50pt, height: auto)
#h(99%) 🏳️‍🌈
🏳️‍🌈