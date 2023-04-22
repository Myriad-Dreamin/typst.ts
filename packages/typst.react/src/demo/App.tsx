import { useEffect, useState } from 'react';
import { TypstDocument } from '../lib';

export const App = () => {
  const [artifact, setArtifact] = useState<string>('');

  const getArtifactJson = async () => {
    const response = await fetch('/main.artifact.json').then(response => response.text());

    setArtifact(response);
  };

  useEffect(() => {
    getArtifactJson();
  }, []);

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
        Demo: Embed Your Typst Document in React
      </h1>
      <TypstDocument fill="#343541" artifact={artifact} />
    </div>
  );
};
