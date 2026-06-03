
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// `align` is block-level and should interrupt a list.
#show list: [List]
- a
- b
#align(right)[- i]
- j