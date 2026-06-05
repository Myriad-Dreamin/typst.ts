
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)

```cpp
#include <iostream>

int main() {
  std::cout << "Hello, world!";
}
```