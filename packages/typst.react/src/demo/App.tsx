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
      <h1>Embed Your Typst Document React Demo</h1>
      <TypstDocument fill="#343541" artifact={artifact} />
    </div>
  );
};
