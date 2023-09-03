
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test setting a column gutter and more than two columns.
#set page(height: 3.25cm, width: 7.05cm, columns: 3)
#set columns(gutter: 30pt)

#rect(width: 100%, height: 2.5cm, fill: conifer) #parbreak()
#rect(width: 100%, height: 2cm, fill: eastern) #parbreak()
#circle(fill: eastern)
