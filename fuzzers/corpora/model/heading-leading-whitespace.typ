
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that leading whitespace and comments don't matter.
#test[= h][=        h]
#test[= h][=   /**/  /**/   h]
#test[= h][=   /*
comment spans lines
*/   h]