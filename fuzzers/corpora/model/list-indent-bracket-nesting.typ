
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test list indent nesting behavior when directly at a starting bracket.

#let indented = {
  [- indented
  - less
  ]
  [- indented
   - same
  - then less
   - then same
  ]
  [- indented
    - more
   - then same
  - then less
  ]
}

#let item = list.item
#let manual = {
    {
      item[indented]; [ ]
      item[less]; [ ]
    }
    {
      item[indented]; [ ]
      item[same]; [ ]
      item[then less #{
        item[then same]
      }]; [ ]
    }
    {
      item[indented #{
        item[more]
      }]; [ ]
      item[then same]; [ ]
      item[then less]; [ ]
    }
}

#test(indented, manual)