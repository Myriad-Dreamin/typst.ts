// import html from 'index.typ?html';
import { title, description, body, frontmatter } from 'main.typ?parts';

console.log(title, description, frontmatter);

document.body.innerHTML = body;
