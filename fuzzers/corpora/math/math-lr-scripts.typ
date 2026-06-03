
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test interactions with script attachments.
$ lr(size: #3em, |)_a^b lr(size: #3em, zws|)_a^b
  lr(size: #3em, [x])_0^1 [x]_0^1
  lr(size: #1em, lr(size: #10em, [x]))_0^1 $