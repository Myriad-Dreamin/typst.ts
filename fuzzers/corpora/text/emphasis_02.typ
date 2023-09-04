
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Adjusting the delta that strong applies on the weight.
Normal

#set strong(delta: 300)
*Bold*

#set strong(delta: 150)
*Medium* and *#[*Bold*]*
