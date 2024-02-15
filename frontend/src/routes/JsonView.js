import { useEffect, useState } from 'react';
import { AnimatedTree } from 'react-tree-graph';
import ir_view from '../helpers/ir_view';
import styles from '../styles/tree.css';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import { TreeView } from '@mui/x-tree-view/TreeView';
import { TreeItem } from '@mui/x-tree-view/TreeItem';
import { Grid, JsonInput, Tabs } from '@mantine/core';
import { IconBinaryTree, IconListTree } from '@tabler/icons-react';

const iconStyle = { width: '1.2rem', height: '1.2rem' };

const generateNodeIds = (node, nodeId = 0, ids = []) => {
  ids.push(nodeId.toString());
  if (Array.isArray(node.children)) {
    node.children.forEach((child, index) =>
      generateNodeIds(child, nodeId * 10 + index + 1, ids),
    );
  }
  return ids;
};

function JsonView() {
  const [tree, setTree] = useState(null);
  const [inputJson, setInputJson] = useState('');
  const [expandedNodeIds, setExpandedNodeIds] = useState([]);

  // FIXME: this should be a component
  const renderTree = (node, nodeId = 0) => {
    return (
      <TreeItem
        key={nodeId}
        nodeId={nodeId.toString()}
        label={node.name}
        sx={{
          '& .MuiTreeItem-label': {
            textAlign: 'left',
            fontSize: '1rem', // Adjust the font size as needed
          },
          '& .MuiTreeItem-content': {
            alignItems: 'flex-start',
          },
        }}
      >
        {Array.isArray(node.children)
          ? node.children.map((child, index) =>
              renderTree(child, nodeId * 10 + index + 1),
            )
          : null}
      </TreeItem>
    );
  };

  const handleNodeToggle = (event, nodeIds) => {
    setExpandedNodeIds(nodeIds);
  };

  useEffect(() => {
    try {
      const json = JSON.parse(inputJson);
      setTree(null);
      requestAnimationFrame(() => {
        const generatedTree = ir_view.generateTree(json);
        setTree(generatedTree);
        // Initialize expandedNodeIds with all node IDs when the tree is first generated
        setExpandedNodeIds(generateNodeIds(generatedTree));
      });
    } catch (e) {
      // Error is already handled by input component
    }
  }, [inputJson]);

  return (
    <Grid>
      <Grid.Col span={6}>
        <JsonInput
          label="JSON IR"
          formatOnBlur
          value={inputJson}
          onChange={(value) => {
            setInputJson(value);
          }}
          styles={{
            input: { minHeight: 300 },
          }}
        />
      </Grid.Col>
      <Grid.Col span={6}>
        <Tabs defaultValue="ast">
          <Tabs.List>
            <Tabs.Tab
              value="ast"
              leftSection={<IconListTree style={iconStyle} />}
            >
              AST
            </Tabs.Tab>
            <Tabs.Tab
              value="tree"
              leftSection={<IconBinaryTree style={iconStyle} />}
            >
              AST Tree
            </Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="ast">
            {tree && (
              <TreeView
                sx={{
                  flexGrow: 1,
                  overflowY: 'auto',
                  '& .MuiTreeItem-root': {
                    textAlign: 'left', // Aligns the TreeItem text to left
                  },
                }}
                aria-label="IR Tree View"
                defaultCollapseIcon={<ExpandMoreIcon />}
                defaultExpandIcon={<ChevronRightIcon />}
                expanded={expandedNodeIds}
                onNodeToggle={handleNodeToggle}
              >
                {renderTree(tree)}
              </TreeView>
            )}
          </Tabs.Panel>

          <Tabs.Panel value="tree">
            {tree && (
              <AnimatedTree
                data={tree}
                height={700}
                width={500}
                svgProps={{ transform: 'rotate(90)' }}
                textProps={{ fill: 'white', transform: 'rotate(270)' }}
                className={styles}
                duration={750}
                steps={20}
              />
            )}
          </Tabs.Panel>
        </Tabs>
      </Grid.Col>
    </Grid>
  );
}

export default JsonView;
