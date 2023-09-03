
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `columns` function.
#set page(width: auto)

#rect(width: 180pt, height: 100pt, inset: 8pt, columns(2, [
    A special plight has befallen our document.
    Columns in text boxes reigned down unto the soil
    to waste a year's crop of rich layouts.
    The columns at least were graciously balanced.
]))
