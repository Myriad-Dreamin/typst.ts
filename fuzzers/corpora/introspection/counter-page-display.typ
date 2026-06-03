
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Counter display should use numbering from style chain.
#set page(
  numbering: "A",
  margin: (bottom: 20pt),
  footer: context align(center, counter(page).display())
)