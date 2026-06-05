
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test 2d alignment 'vertical' field.
#test((start + top).y, top)
#test((end + top).y, top)
#test((left + top).y, top)
#test((right + top).y, top)
#test((center + top).y, top)
#test((start + bottom).y, bottom)
#test((end + bottom).y, bottom)
#test((left + bottom).y, bottom)
#test((right + bottom).y, bottom)
#test((center + bottom).y, bottom)
#test((start + horizon).y, horizon)
#test((end + horizon).y, horizon)
#test((left + horizon).y, horizon)
#test((right + horizon).y, horizon)
#test((center + horizon).y, horizon)
#test((top + start).y, top)
#test((bottom + end).y, bottom)
#test((horizon + center).y, horizon)