
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test prime symbols don't raise the superscript position
$
  sqrt(f)/f
  sqrt(f^2)/f^2
  sqrt(f'^2)/f'^2
  sqrt(f''_n^2)/f''^2_n
$