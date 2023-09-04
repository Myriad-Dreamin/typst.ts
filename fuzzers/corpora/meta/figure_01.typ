
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


// Testing figures with tables.
#figure(
  table(
    columns: 2,
    [Second cylinder],
    image("/assets/files/cylinder.svg"),
  ),
  caption: "A table containing images."
) <fig-image-in-table>
