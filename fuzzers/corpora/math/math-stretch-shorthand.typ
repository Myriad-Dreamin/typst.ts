
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stretch when base is given with shorthand.
$stretch(||, size: #2em)$
$stretch(\(, size: #2em)$
$stretch(⟧, size: #2em)$
$stretch(|, size: #2em)$
$stretch(->, size: #2em)$
$stretch(↣, size: #2em)$