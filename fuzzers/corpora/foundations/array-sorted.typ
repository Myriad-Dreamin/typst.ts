
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `sorted` method.
#test(().sorted(), ())
#test(().sorted(key: x => x), ())
#test(((true, false) * 10).sorted(), (false,) * 10 + (true,) * 10)
#test(("it", "the", "hi", "text").sorted(), ("hi", "it", "text", "the"))
#test(("I", "the", "hi", "text").sorted(key: x => x), ("I", "hi", "text", "the"))
#test(("I", "the", "hi", "text").sorted(key: x => x.len()), ("I", "hi", "the", "text"))
#test((2, 1, 3, 10, 5, 8, 6, -7, 2).sorted(), (-7, 1, 2, 2, 3, 5, 6, 8, 10))
#test((2, 1, 3, -10, -5, 8, 6, -7, 2).sorted(key: x => x), (-10, -7, -5, 1, 2, 2, 3, 6, 8))
#test((2, 1, 3, -10, -5, 8, 6, -7, 2).sorted(key: x => x * x), (1, 2, 2, 3, -5, 6, -7, 8, -10))
#test(("I", "the", "hi", "text").sorted(by: (x, y) => x.len() < y.len()), ("I", "hi", "the", "text"))
#test(("I", "the", "hi", "text").sorted(key: x => x.len(), by: (x, y) => y < x), ("text", "the", "hi", "I"))