
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test script-script in a fraction.
$ 1/(x^A) $
#[#set text(size:18pt); $1/(x^A)$] vs. #[#set text(size:14pt); $x^A$]
