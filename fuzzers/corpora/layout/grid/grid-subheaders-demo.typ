
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 15.2em)
#table(
  columns: 2,
  align: center,
  table.header(
    table.cell(colspan: 2)[*Regional User Data*],
  ),
  table.header(
    level: 2,
    table.cell(colspan: 2)[*Germany*],
    [*Username*], [*Joined*]
  ),
  [john123], [2024],
  [rob8], [2025],
  [joe1], [2025],
  [joe2], [2025],
  [martha], [2025],
  [pear], [2025],
  table.header(
    level: 2,
    table.cell(colspan: 2)[*United States*],
    [*Username*], [*Joined*]
  ),
  [cool4], [2023],
  [roger], [2023],
  [bigfan55], [2022]
)