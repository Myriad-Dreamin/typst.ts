
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// A mixture of attachment positioning schemes.
$
attach(a, tl: u),   attach(a, tr: v),   attach(a, bl: x),
attach(a, br: y),   limits(a)^t,        limits(a)_b \

attach(a, tr: v, t: t),
attach(a, tr: v, br: y),
attach(a, br: y, b: b),
attach(limits(a), b: b, bl: x),
attach(a, tl: u, bl: x),
attach(limits(a), t: t, tl: u) \

attach(a, tl: u, tr: v),
attach(limits(a), t: t, br: y),
attach(limits(a), b: b, tr: v),
attach(a, bl: x, br: y),
attach(limits(a), b: b, tl: u),
attach(limits(a), t: t, bl: u),
limits(a)^t_b \

attach(a, tl: u, tr: v, bl: x, br: y),
attach(limits(a), t: t, bl: x, br: y, b: b),
attach(limits(a), t: t, tl: u, tr: v, b: b),
attach(limits(a), tl: u, bl: x, t: t, b: b),
attach(limits(a), t: t, b: b, tr: v, br: y),
attach(a, tl: u, t: t, tr: v, bl: x, b: b, br: y)
$
