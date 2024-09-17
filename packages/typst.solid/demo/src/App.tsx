import type { Component } from 'solid-js';
import { createResource, Show } from 'solid-js'
import { TypstDocument } from 'typst-ts-solid';

export const App = (artifact: Uint8Array) => {
  const getArtifactData = async () => {
    const response = await fetch(
      // get pre-compiled file
      'https://myriad-dreamin.github.io/typst.ts/docs/readme.artifact.sir.in'
    ).then(response => response.arrayBuffer());

    return (new Uint8Array(response));
  };
  const [vector] = createResource(getArtifactData);

  return (
    <div>
      <Show when={vector()} fallback={<h1>Loading...</h1>}>
        <TypstDocument fill="#343541" artifact={vector()} />
      </Show>
    </div>
  );
};
export default App;
