
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#test(`foo`, `foo`)
#assert.ne(`foo`, `bar`)