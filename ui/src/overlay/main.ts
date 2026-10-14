import { mount } from 'svelte';
import Overlay from './Overlay.svelte';
import '../styles/tokens.css';
import '../styles/overlay.css';

const target = document.getElementById('overlay');
if (!target) throw new Error('missing #overlay mount point');

export default mount(Overlay, { target });
