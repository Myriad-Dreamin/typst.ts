
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Two-dimensional arguments in math. More tests can be found in `mat.typ`.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(a;b)$, "arguments(([a],), ([b],))")
#check($args(a,b;c)$, "arguments(([a], [b]), ([c],))")
#check($args(a,b;c,d;e,f)$, "arguments(([a], [b]), ([c], [d]), ([e], [f]))")
#check($args( a; b; )$, "arguments(([a],), ([b],))")
#check($args(a;  ; c)$, "arguments(([a],), ([],), ([c],))")
#check($args(a b,/**/; b)$, "arguments((sequence([a], [ ], [b]), []), ([b],))")
#check($args(a/**/b, ; b)$, "arguments((sequence([a], [b]), []), ([b],))")
#check($args( ;/**/a/**/b/**/; )$, "arguments(([],), (sequence([a], [b]),))")
#check($args( ; , ; )$, "arguments(([],), ([], []))")
#check($args(/**/; // funky whitespace/trivia
    ,   /**/  ;/**/)$, "arguments(([],), ([], []))")