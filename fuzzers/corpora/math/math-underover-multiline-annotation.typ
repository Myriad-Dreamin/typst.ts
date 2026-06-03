
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that multiline annotations do not change the baseline.
$ S = overbrace(beta (alpha) S I, "one line")
    - overbrace(mu (N), "two" \  "line") $
$ S = underbrace(beta (alpha) S I, "one line")
    - underbrace(mu (N), "two" \  "line") $