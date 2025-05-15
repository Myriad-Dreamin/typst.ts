import { useState } from 'react';
import { TypstDocument } from './TypstDocument';
import "./App.css";

export const App = () => {
  const [source, setSource] = useState<string>(`
#set text(fill: white);
= Heading
hello
`.trim());

  return (
    <div>
      <h1
        style={{
          color: 'white',
          fontSize: '20px',
          fontFamily: `'Garamond', sans-serif`,
          margin: '20px',
        }}
      >
        Demo: Run Typst Compiler in Browser
      </h1>
      <textarea className='demo-input' value={source} onChange={it => setSource(it.target.value)}></textarea>
      <TypstDocument fill="#343541" source={source} />
    </div>
  );
};
