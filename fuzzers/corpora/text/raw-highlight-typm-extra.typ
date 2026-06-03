
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Math highlighting for strings, alignments, shorthands, and named args.
#set page(width: auto)
```typm
"string" - + * ::= & \
|=> & [|define(x: #y, x::= y)|]
```