
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that label only works within one content block.
#show <strike>: strike
// Warning: 13-21 label `<strike>` is not attached to anything
*This is* #[<strike>] *protected.*
*This is not.* <strike>