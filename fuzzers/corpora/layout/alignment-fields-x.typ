
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test 2d alignment 'horizontal' field.
#test((start + top).x, start)
#test((end + top).x, end)
#test((left + top).x, left)
#test((right + top).x, right)
#test((center + top).x, center)
#test((start + bottom).x, start)
#test((end + bottom).x, end)
#test((left + bottom).x, left)
#test((right + bottom).x, right)
#test((center + bottom).x, center)
#test((start + horizon).x, start)
#test((end + horizon).x, end)
#test((left + horizon).x, left)
#test((right + horizon).x, right)
#test((center + horizon).x, center)
#test((top + start).x, start)
#test((bottom + end).x, end)
#test((horizon + center).x, center)