
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test equality operators.

// Most things compare by value.
#test(1 == "hi", false)
#test(1 == 1.0, true)
#test(30% == 30% + 0cm, true)
#test(1in == 0% + 72pt, true)
#test(30% == 30% + 1cm, false)
#test("ab" == "a" + "b", true)
#test(() == (1,), false)
#test((1, 2, 3) == (1, 2.0) + (3,), true)
#test((:) == (a: 1), false)
#test((a: 2 - 1.0, b: 2) == (b: 2, a: 1), true)
#test("a" != "a", false)
#test(decimal("1.234") == decimal("1.23400000000"), true)
#test(235 == decimal("235.0"), true)

// Functions compare by identity.
#test(test == test, true)
#test((() => {}) == (() => {}), false)

// Content compares field by field.
#let t = [a]
#test(t == t, true)
#test([] == [], true)
#test([a] == [a], true)
#test(grid[a] == grid[a], true)
#test(grid[a] == grid[b], false)