## Installation

```bash
yarn add @myriaddreamin/typst.svelte
```

## Usage
```svelte
<script lang="ts">
    import Typst from "@myriaddreamin/typst.svelte"

	const mainContent = `
                = Hello Typst
        `;
</script>

<Typst {mainContent}></Typst>
```
