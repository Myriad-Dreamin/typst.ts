
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test variation selectors for scr and cal.
$cal(A) scr(A) bold(cal(O)) scr(bold(O))$

#show math.equation: set text(font: "Noto Sans Math")
$scr(E) cal(E) bold(scr(Y)) cal(bold(Y))$