
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The page number is not displayed on the page. Instead, it is only computed to
// be embedded in the PDF metadata so the error is triggered in `typst-pdf`
// instead of `typst-layout`. For now, we ignore it and generate the PDF anyway,
// without using the user-provided page numbering.
#set page(numbering: "①", footer: none)
#counter(page).update(100)
Hello