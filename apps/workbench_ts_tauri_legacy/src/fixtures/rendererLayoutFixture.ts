export type InspectorNode = {
  id: string;
  kind: string;
  rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  children: InspectorNode[];
};

export const layoutFixture: InspectorNode = {
  id: 'root',
  kind: 'canvas',
  rect: { x: 0, y: 0, width: 960, height: 540 },
  children: [
    {
      id: 'header',
      kind: 'panel',
      rect: { x: 0, y: 0, width: 960, height: 64 },
      children: []
    },
    {
      id: 'sidebar',
      kind: 'panel',
      rect: { x: 0, y: 64, width: 220, height: 420 },
      children: []
    },
    {
      id: 'content',
      kind: 'panel',
      rect: { x: 220, y: 64, width: 740, height: 420 },
      children: [
        {
          id: 'card_a',
          kind: 'card',
          rect: { x: 244, y: 88, width: 320, height: 160 },
          children: []
        },
        {
          id: 'card_b',
          kind: 'card',
          rect: { x: 588, y: 88, width: 320, height: 160 },
          children: []
        }
      ]
    },
    {
      id: 'footer',
      kind: 'panel',
      rect: { x: 0, y: 484, width: 960, height: 56 },
      children: []
    }
  ]
};

export function flattenNodes(root: InspectorNode): InspectorNode[] {
  const result: InspectorNode[] = [];
  function traverse(node: InspectorNode) {
    result.push(node);
    for (const child of node.children) {
      traverse(child);
    }
  }
  traverse(root);
  return result;
}

export function findNode(root: InspectorNode, id: string): InspectorNode | undefined {
  if (root.id === id) return root;
  for (const child of root.children) {
    const found = findNode(child, id);
    if (found) return found;
  }
  return undefined;
}
