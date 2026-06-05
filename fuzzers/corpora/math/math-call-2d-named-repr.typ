
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args(..body) = (body.pos(), body.named())
#let check(it, r) = test-repr(it.body.text, r)
#check($args(a: b)$, "((), (a: [b]))")
#check($args(1, 2; 3, 4)$, "((([1], [2]), ([3], [4])), (:))")
#check($args(a: b, 1, 2; 3, 4)$, "((([1], [2]), ([3], [4])), (a: [b]))")
#check($args(1, a: b, 2; 3, 4)$, "(([1], ([2],), ([3], [4])), (a: [b]))")
#check($args(1, 2, a: b; 3, 4)$, "(([1], [2], (), ([3], [4])), (a: [b]))")
#check($args(1, 2; a: b, 3, 4)$, "((([1], [2]), ([3], [4])), (a: [b]))")
#check($args(1, 2; 3, a: b, 4)$, "((([1], [2]), [3], ([4],)), (a: [b]))")
#check($args(1, 2; 3, 4, a: b)$, "((([1], [2]), [3], [4]), (a: [b]))")
#check($args(a: b, 1, 2, 3, c: d)$, "(([1], [2], [3]), (a: [b], c: [d]))")
#check($args(1, 2, 3; a: b)$, "((([1], [2], [3]),), (a: [b]))")
#check($args(a-b: a,, e:f;; d)$, "(([], (), ([],), ([d],)), (a-b: [a], e: [f]))")
#check($args(a: b, ..#range(0, 4))$, "((0, 1, 2, 3), (a: [b]))")