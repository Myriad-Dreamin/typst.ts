
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that widow/orphan prevention doesn't unnecessarily move things
// to another page.
#set page(width: 16cm)
#block(height: 30pt, fill: aqua, columns(2, lorem(19)))