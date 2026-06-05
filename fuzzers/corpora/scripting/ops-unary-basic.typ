
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test math operators.

// Test plus and minus.
#for v in (1, 3.14, decimal("12.43"), 12pt, 45deg, 90%, 13% + 10pt, 6.3fr) {
  // Test plus.
  test(+v, v)

  // Test minus.
  test(-v, -1 * v)
  test(--v, v)

  // Test combination.
  test(-++ --v, -v)
}

#test(-(4 + 2), 6-12)

// Addition.
#test(2 + 4, 6)
#test("a" + "b", "ab")
#test("a" + if false { "b" }, "a")
#test("a" + if true { "b" }, "ab")
#test(13 * "a" + "bbbbbb", "aaaaaaaaaaaaabbbbbb")
#test((1, 2) + (3, 4), (1, 2, 3, 4))
#test((a: 1) + (b: 2, c: 3), (a: 1, b: 2, c: 3))