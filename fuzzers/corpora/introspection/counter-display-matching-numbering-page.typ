
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Counter display should use the page numbering at the location.
#set page(numbering: "(i)", margin: (bottom: 20pt))
#metadata(none) <first>
Second page:
#context counter(page).display(at: <second>)

#set page(
  numbering: "A",
  footer: align(center, {
    "Page: "
    context counter(page).display()
  }),
)
#metadata(none) <second>
First page:
#context counter(page).display(at: <first>)