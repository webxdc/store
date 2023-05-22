/* @refresh reload */
import { render } from 'solid-js/web';

import './index.sass';
import AppStore from './appstore';
import Review from './review'
import "virtual:uno.css"
import '@unocss/reset/tailwind.css'

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got mispelled?',
  );
}

console.log(import.meta.env)

if (import.meta.env.VITE_APPSTORE) {
  render(() => <AppStore />, root!);
} else {
  render(() => <Review />, root!);
}

