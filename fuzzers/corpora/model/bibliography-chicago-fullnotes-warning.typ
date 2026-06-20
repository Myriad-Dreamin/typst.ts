
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test warning for deprecated alias.
// Warning: 47-66 style `"chicago-fullnotes"` has been deprecated in favor of `"chicago-notes"`
#bibliography("/assets/bib/works.bib", style: "chicago-fullnotes", title: none)