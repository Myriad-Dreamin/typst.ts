
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 200pt)
#show raw.line: set text(fill: red)

```py
import numpy as np

def f(x):
    return x**2

x = np.linspace(0, 10, 100)
y = f(x)

print(x)
print(y)
```
