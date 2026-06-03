
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let nums = range(0, 2).map(i => (i, i+1))
$ mat(..nums, delim: "|",)
  mat(..nums; delim: "|",) $
$ mat(..nums) mat(..nums;) \
  mat(..nums;,) mat(..nums,) $