# @myriaddreamin/typst.solid


## Usage

```ts
import { TypstDocument } from "@myriaddreamin/typst.solid";
import { createResource } from "solid-js";

export const App = (artifact: Uint8Array) => {
  const getArtifactData = async () => {
    const response = await fetch(
      "http://localhost:3000/readme.artifact.sir.in"
    ).then((response) => response.arrayBuffer());

    return new Uint8Array(response);
  };
  const [vec] = createResource(getArtifactData);

  return (
    <div>
      <h1>Demo: Embed Your Typst Document in Solid </h1>
      <TypstDocument fill="#343541" artifact={vec()} />
    </div>
  );
};
```

## Documentation

