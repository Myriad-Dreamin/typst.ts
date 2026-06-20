
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The outline thing is just a roundabout way to force a cast from symbol to
// function...
#test(repr(outline(indent: sym.chevron.l.curly).indent), "(..) => ..")