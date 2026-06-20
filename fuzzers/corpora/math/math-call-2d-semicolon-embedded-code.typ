
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// If a semicolon directly follows an embedded code expression, it terminates
// the code expression instead of indicating 2d arguments.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(#false;)$, "arguments(false)")
#check($args("a" #"b";)$, "arguments(sequence([a], [ ], [b]))")
#check($args(#true ;)$, "arguments((true,))")
#check($args(#true;;)$, "arguments((true,))")