
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test columns for a sized page.
#set page(height: 5cm, width: 7.05cm, columns: 2)

Lorem ipsum dolor sit amet is a common blind text
and I again am in need of filling up this page
#align(bottom, rect(fill: eastern, width: 100%, height: 12pt))
#colbreak()

so I'm returning to this trusty tool of tangible terror.
Sure, it is not the most creative way of filling up
a page for a test but it does get the job done.
