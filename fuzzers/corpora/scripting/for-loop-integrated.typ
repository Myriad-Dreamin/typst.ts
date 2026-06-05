
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let out = ()

// Values of array.
#for v in (1, 2, 3) {
  out += (v,)
}

// Indices and values of array.
#for (i, v) in ("1", "2", "3").enumerate() {
  test(repr(i + 1), v)
}

// Pairs of dictionary.
#for v in (a: 4, b: 5) {
  out += (v,)
}

// Keys and values of dictionary.
#for (k, v) in (a: 6, b: 7) {
  out += (k,)
  out += (v,)
}

#test(out, (1, 2, 3, ("a", 4), ("b", 5), "a", 6, "b", 7))

// Grapheme clusters of string.
#let first = true
#let joined = for c in "abc👩‍👩‍👦‍👦" {
  if not first { ", " }
  first = false
  c
}

#test(joined, "a, b, c, 👩‍👩‍👦‍👦")

// Return value.
#test(for v in "" [], none)
#test(type(for v in "1" []), content)