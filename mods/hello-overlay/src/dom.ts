// Tiny DOM helper: all text goes through textContent, never innerHTML.

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: { className?: string; text?: string; onclick?: () => void } = {},
  ...children: (Node | null)[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (props.className) node.className = props.className;
  if (props.text !== undefined) node.textContent = props.text;
  if (props.onclick) node.addEventListener('click', props.onclick);
  for (const child of children) if (child) node.append(child);
  return node;
}
