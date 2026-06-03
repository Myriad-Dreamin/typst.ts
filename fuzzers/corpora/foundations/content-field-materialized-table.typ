
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that fields from set rules are materialized into the element before
// a show rule runs.
#set table(columns: (10pt, auto))
#show table: it => it.columns
#table[A][B][C][D]