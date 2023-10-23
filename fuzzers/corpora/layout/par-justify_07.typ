
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that justification doesn't break code blocks

#set par(justify: true)

```cpp
int main() {
  printf("Hello world\n");
  return 0;
}
```

