
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that the period is trimmed.
#show link: underline
https://a.b.?q=%10#. \
Wahttp://link \
Nohttps:\//link \
Nohttp\://comment
