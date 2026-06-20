
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The marker doesn't move as the list body expands and aligns itself.
// However, with `block`, the list body does not expand, so the marker is also
// aligned.
+ a
+ b
#align(right)[+ c]
#align(right, block[+ d])