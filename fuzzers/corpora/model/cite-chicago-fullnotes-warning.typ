
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test warning for deprecated alias.
// Warning: 24-43 style `"chicago-fullnotes"` has been deprecated in favor of `"chicago-notes"`
#cite(<netwok>, style: "chicago-fullnotes")
#bibliography("/assets/bib/works.bib")