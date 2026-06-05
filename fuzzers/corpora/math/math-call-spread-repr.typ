
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(..#range(0, 4).chunks(2))$, "arguments((0, 1), (2, 3))")
#check($#args(range(1, 5).chunks(2))$, "arguments(((1, 2), (3, 4)))")
#check($#args(..range(1, 5).chunks(2))$, "arguments((1, 2), (3, 4))")
#check($args(#(..range(2, 6).chunks(2)))$, "arguments(((2, 3), (4, 5)))")
#let nums = range(0, 4).chunks(2)
#check($args(..nums)$, "arguments((0, 1), (2, 3))")
#check($args(..nums;)$, "arguments(((0, 1), (2, 3)))")
#check($args(..nums, ..nums)$, "arguments((0, 1), (2, 3), (0, 1), (2, 3))")
#check($args(..nums, 4, 5)$, "arguments((0, 1), (2, 3), [4], [5])")
#check($args(..nums, ..#range(4, 6))$, "arguments((0, 1), (2, 3), 4, 5)")
#check($args(..nums, #range(4, 6))$, "arguments((0, 1), (2, 3), (4, 5))")
#check($args(..nums, 1, 2; 3, 4)$, "arguments(((0, 1), (2, 3), [1], [2]), ([3], [4]))")
#check($args(1, 2; ..nums)$, "arguments(([1], [2]), ((0, 1), (2, 3)))")
#check($args(1, 2; 3, 4)$, "arguments(([1], [2]), ([3], [4]))")
#check($args(1, 2; 3, 4; ..#range(5, 7))$, "arguments(([1], [2]), ([3], [4]), (5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7))$, "arguments(([1], [2]), ([3], [4], 5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7);)$, "arguments(([1], [2]), ([3], [4], 5, 6))")
#check($args(1, 2; 3, 4, ..#range(5, 7),)$, "arguments(([1], [2]), ([3], [4], 5, 6))")