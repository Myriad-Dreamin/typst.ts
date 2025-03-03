// import html from 'index.typ?html';
import { title, description, body } from 'template.typ?parts';

console.log(title, description);

document.body.innerHTML = body;
