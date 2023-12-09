import assert from 'node:assert/strict'
import test from 'node:test'
import rehypeTypst from 'rehype-typst'
import rehypeStringify from 'rehype-stringify'
import remarkMath from 'remark-math'
import remarkParse from 'remark-parse'
import remarkRehype from 'remark-rehype'
import { unified } from 'unified'

const test_code = `
## test $a+b^2$

We have:
$$
F_n = round(1 / sqrt(5) phi.alt^n), quad
  phi.alt = (1 + sqrt(5)) / 2
$$

But that's not all, we also have:
$$
F_n = round(1 / sqrt(5) phi^n), quad
  phi = (1 - sqrt(5)) / 2
$$

In general,$ F_n = round(1 / sqrt(5) phi^n) $ and $ F_n = round(1 / sqrt(5) phi.alt^n) $. Also note that $1+e/sqrt(sqrt(a/c)/(e + c +a/b))$ and $F_n = round(1 / sqrt(5) phi^n), quad phi = (1 - sqrt(5)) / 2$ 123123213 $ F_n = round(1 / sqrt(5) phi.alt^n) $ dsfne3 0difdk 3434klk $1+1/1/1/1/sqrt(1)$
`

unified()
  .use(remarkParse)
  .use(remarkMath)
  .use(remarkRehype)
  .use(rehypeTypst)
  .use(rehypeStringify)
  .process(test_code, (err, file) => {
    console.log(String(file))
  })
