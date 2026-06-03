
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Primarily to ensure that it does not crash in PDF export.
#image("/assets/images/f2t.jpg", width: 0pt, height: 0pt)