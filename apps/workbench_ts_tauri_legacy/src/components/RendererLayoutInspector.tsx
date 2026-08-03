import { useState } from 'react';
import { layoutFixture, flattenNodes, type InspectorNode } from '../fixtures/rendererLayoutFixture';
import './RendererLayoutInspector.css';

export function RendererLayoutInspector() {
  const [selectedNodeId, setSelectedNodeId] = useState<string>('content');
  const root = layoutFixture;
  const flatNodes = flattenNodes(root);
  const selectedNode = flatNodes.find(n => n.id === selectedNodeId);

  const handleNodeClick = (id: string) => {
    setSelectedNodeId(id);
  };

  const renderTree = (node: InspectorNode, depth = 0) => {
    const isSelected = node.id === selectedNodeId;
    return (
      <div key={node.id}>
        <div 
          className={`tree-node ${isSelected ? 'selected' : ''}`}
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => handleNodeClick(node.id)}
        >
          <span className="tree-node-id">{node.id}</span>
          <span className="tree-node-kind">{node.kind}</span>
        </div>
        {node.children.map(child => renderTree(child, depth + 1))}
      </div>
    );
  };

  const renderRect = (node: InspectorNode) => {
    const isSelected = node.id === selectedNodeId;
    
    // For proper scaling, we can use a container with a specific aspect ratio, 
    // or just assume a fixed scale. Let's use a percentage-based approach for flexibility.
    const rootWidth = root.rect.width;
    const rootHeight = root.rect.height;
    
    const left = (node.rect.x / rootWidth) * 100;
    const top = (node.rect.y / rootHeight) * 100;
    const width = (node.rect.width / rootWidth) * 100;
    const height = (node.rect.height / rootHeight) * 100;

    return (
      <div
        key={node.id}
        className={`canvas-rect ${isSelected ? 'selected' : ''}`}
        style={{
          left: `${left}%`,
          top: `${top}%`,
          width: `${width}%`,
          height: `${height}%`,
        }}
        onClick={(e) => {
          e.stopPropagation();
          handleNodeClick(node.id);
        }}
      >
        <div className="canvas-rect-label">{node.id}</div>
        {node.children.map(child => renderRect(child))}
      </div>
    );
  };

  return (
    <div className="renderer-layout-inspector">
      <div className="inspector-header">
        <h2>Renderer / Layout Inspector</h2>
      </div>
      
      <div className="inspector-body">
        <div className="panel node-tree-panel">
          <h3 className="panel-title">Node Tree</h3>
          <div className="node-tree-content">
            {renderTree(root)}
          </div>
        </div>
        
        <div className="panel canvas-panel">
          <h3 className="panel-title">Layout Canvas</h3>
          <div className="canvas-container">
            <div className="canvas-aspect-ratio" style={{ aspectRatio: `${root.rect.width} / ${root.rect.height}` }}>
              {renderRect(root)}
            </div>
          </div>
        </div>
        
        <div className="panel details-panel">
          <h3 className="panel-title">Details</h3>
          <div className="details-content">
            {selectedNode ? (
              <div className="node-details">
                <div className="detail-row"><span className="detail-key">id:</span> <span className="detail-value">{selectedNode.id}</span></div>
                <div className="detail-row"><span className="detail-key">kind:</span> <span className="detail-value">{selectedNode.kind}</span></div>
                <div className="detail-row"><span className="detail-key">x:</span> <span className="detail-value">{selectedNode.rect.x}</span></div>
                <div className="detail-row"><span className="detail-key">y:</span> <span className="detail-value">{selectedNode.rect.y}</span></div>
                <div className="detail-row"><span className="detail-key">width:</span> <span className="detail-value">{selectedNode.rect.width}</span></div>
                <div className="detail-row"><span className="detail-key">height:</span> <span className="detail-value">{selectedNode.rect.height}</span></div>
                <div className="detail-row"><span className="detail-key">children:</span> <span className="detail-value">{selectedNode.children.length}</span></div>
              </div>
            ) : (
              <div className="no-selection">No node selected</div>
            )}
          </div>
        </div>
      </div>
      
      <div className="inspector-footer">
        Prototype only — fixture data, no backend connection.
      </div>
    </div>
  );
}
