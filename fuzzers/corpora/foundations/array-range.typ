
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `range` function.
#test(range(4), (0, 1, 2, 3))
#test(range(1, 4), (1, 2, 3))
#test(range(-4, 2), (-4, -3, -2, -1, 0, 1))
#test(range(10, 5), ())
#test(range(10, step: 3), (0, 3, 6, 9))
#test(range(1, 4, step: 1), (1, 2, 3))
#test(range(1, 8, step: 2), (1, 3, 5, 7))
#test(range(5, 2, step: -1), (5, 4, 3))
#test(range(10, 0, step: -3), (10, 7, 4, 1))

#test(range(inclusive: true, 0, 0), (0,))
#test(range(inclusive: true, -10, -8), (-10, -9, -8))
#test(range(inclusive: true, -2, 4, step: 2), (-2, 0, 2, 4))
#test(range(inclusive: true, 5, 2, step: -1), (5, 4, 3, 2))
#test(range(inclusive: true, 0, -2, step: -1), (0, -1, -2))

// The user should be able to reach these values.
#test(range(int.max - 2, int.max), (int.max - 2, int.max - 1))
#test(range(int.min + 2, int.min, step: -1), (int.min + 2, -int.max))
#test(range(inclusive: true, int.max - 2, int.max), (int.max - 2, int.max - 1, int.max))
#test(range(inclusive: true, int.min + 2, int.min, step: -1), (int.min + 2, -int.max, int.min))

// Stepping would overflow if not caught.
#test(range(2, 3, step: int.max), (2,))
#test(range(-2, -3, step: int.min), (-2,))
#test(range(inclusive: true, int.max - 1, int.max, step: 2), (int.max - 1,))
#test(range(inclusive: true, int.min + 1, int.min, step: -2), (-int.max,))