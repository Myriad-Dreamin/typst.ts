
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Two-dimensional args with positional spreading.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#let nums = range(0, 4).chunks(2)
#check($args(..nums;)$, "arguments(((0, 1), (2, 3)))")
#check($args(..nums; ,)$, "arguments(((0, 1), (2, 3)), ([],))")
#check($args(..nums; ;)$, "arguments(((0, 1), (2, 3)), ([],))")
#check($args(..nums; 1, 2; 3, 4)$, "arguments(((0, 1), (2, 3)), ([1], [2]), ([3], [4]))")
#check($args(..nums, 1, 2; 3, 4)$, "arguments(((0, 1), (2, 3), [1], [2]), ([3], [4]))")
#check($args(1, 2; ..nums)$, "arguments(([1], [2]), ((0, 1), (2, 3)))")
#check($args(1, 2; 3, 4)$, "arguments(([1], [2]), ([3], [4]))")
#check($args(1, 2; 3, 4; ..#range(5, 7))$, "arguments(([1], [2]), ([3], [4]), (5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7))$, "arguments(([1], [2]), ([3], [4], 5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7);)$, "arguments(([1], [2]), ([3], [4], 5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7),)$, "arguments(([1], [2]), ([3], [4], 5, 6))")