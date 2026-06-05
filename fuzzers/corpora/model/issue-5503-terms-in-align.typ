
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// `align` is block-level and should interrupt a `terms`.
#show terms: [Terms]
/ a: a
#align(right)[/ i: i]
/ j: j