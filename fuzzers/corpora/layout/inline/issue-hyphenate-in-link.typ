
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(justify: true)

// The `linebreak()` function accidentally generated out-of-order breakpoints
// for links because it now splits on word boundaries. We avoid the link markup
// syntax because it's show rule interferes.
#"http://creativecommons.org/licenses/by-nc-sa/4.0/"