import React, { SyntheticEvent, useState, useEffect } from 'react';
import { TreeView, TreeItem } from '@mui/x-tree-view';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

interface TreeNode {
  name: string;
  children?: TreeNode[];
}

interface TreeComponentProps {
  tree: TreeNode | null;
}

// Utility function to generate unique node IDs and populate an array with them
const generateNodeIds = (node: TreeNode, prefix = '0'): string[] => {
  const ids: string[] = [prefix];
  if (node.children && node.children.length > 0) {
    node.children.forEach((child, index) => {
      ids.push(...generateNodeIds(child, `${prefix}-${index}`));
    });
  }
  return ids;
};

const TreeComponent: React.FC<TreeComponentProps> = ({ tree }) => {
  const [expandedNodeIds, setExpandedNodeIds] = useState<string[]>([]);

  // Effect to initialize node IDs and expanded state when the tree prop changes
  useEffect(() => {
    if (tree) {
      const initialIds = generateNodeIds(tree);
      setExpandedNodeIds(initialIds); // Expand all nodes initially
      console.log(initialIds)
    }
  }, [tree]);

  // Recursive function to render the tree structure
  const renderTree = (node: TreeNode, nodeId: string = '0'): JSX.Element => (
    <TreeItem key={nodeId} nodeId={nodeId} label={node.name}>
      {node.children?.map((child, index) =>
        renderTree(child, `${nodeId}-${index}`)
      )}
    </TreeItem>
  );

  if (!tree) {
    return <div>No tree data available.</div>;
  }

  return (
    <TreeView
      aria-label="AST Tree"
      defaultCollapseIcon={<ExpandMoreIcon />}
      defaultExpandIcon={<ChevronRightIcon />}
      expanded={expandedNodeIds}
      onNodeToggle={(event: SyntheticEvent, nodeIds: string[]) => setExpandedNodeIds(nodeIds)}
    >
      {tree && renderTree(tree)}
    </TreeView>
  );
};

export default TreeComponent;
