
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Highlighting for Typst math
#set page(width: auto)
```typm
1 + 2/3
sum_(i=1)^n i = (n(n+1))/2
binom(n, k) = n!/(k!(n - k)!)
2 / √(2pi) = sqrt(2) / √pi
3 * (1 - 2) <= #(3 * (1 + 2))
((a+b))/((c)^(d')_(e')_(f)'/(g)'/(h)!)
[\(a+b\)]/{\(c\)^[d']_{e'}_[|f|]'/[g]'/[\|h\|]!}
f_zeta(x), f_zeta(x)/1, f_zeta (x)
pi.alt + pi^arrow.l.long.double - π = ???
"string" - + * ::= & \
|=> & [|define(x-y_z: #1, x::= y; xyz; 0)|]
std.text(op("Red"), fill: red)
#std.text(math.op("Red"), fill: red)
```