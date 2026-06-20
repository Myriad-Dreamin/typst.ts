
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test warning for deprecated alias.
// Warning: 47-87 style `"modern-humanities-research-association"` has been deprecated in favor of `"modern-humanities-research-association-notes"`
#bibliography("/assets/bib/works.bib", style: "modern-humanities-research-association", title: none)