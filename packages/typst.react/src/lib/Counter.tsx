import { useState } from 'react';
import * as abc from '@myriaddreamin/typst.ts';

export const Counter = () => {
  const [count, setCount] = useState(0);
  console.log((window as any).pdfjsLib, abc);
  return (
    <div>
      <h3>Update the count and edit src/App.tsx, state is preserved</h3>
      <button onClick={() => setCount(c => c + 1)}>Count - {count}</button>
    </div>
  );
};
