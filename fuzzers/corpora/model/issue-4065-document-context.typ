
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that we can set document properties based on context.
#show: body => context {
  let all = query(heading)
  let title = if all.len() > 0 { all.first().body }
  set document(title: title)
  body
}

#show heading: none
= Top level