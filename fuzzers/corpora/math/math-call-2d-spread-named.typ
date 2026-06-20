
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Two-dimensional args with named and positional spreading.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#let nums = range(0, 4).chunks(2)
#let dict = (one: 1, two: 2)
#let both = arguments(..nums, ..dict)
#check($args(..nums;)$, "arguments(((0, 1), (2, 3)))")
#check($args(..dict;)$, "arguments(one: 1, two: 2, ())") // Adds an empty array
#check($args(1, ..dict;)$, "arguments(one: 1, two: 2, ([1],))")
#check($args(1, ..dict, 2;)$, "arguments(one: 1, two: 2, ([1], [2]))")
#check($args(1; ..dict, 2;)$, "arguments(one: 1, two: 2, ([1],), ([2],))")
#check($args(1; ..dict; 2;)$, "arguments(one: 1, two: 2, ([1],), (), ([2],))")
#check($args(..nums, ..dict;)$, "arguments(one: 1, two: 2, ((0, 1), (2, 3)))")
#check($args(..both;)$, "arguments(one: 1, two: 2, ((0, 1), (2, 3)))")
#check($args(..nums; ..dict)$, "arguments(one: 1, two: 2, ((0, 1), (2, 3)))")
#check($args(..dict; ..nums)$, "arguments(one: 1, two: 2, (), ((0, 1), (2, 3)))")