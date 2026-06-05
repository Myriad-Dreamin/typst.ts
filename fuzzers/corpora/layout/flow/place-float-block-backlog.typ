
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)
#v(60pt)
#place(top, float: true, rect())
#list(.."ABCDEFGHIJ".clusters())