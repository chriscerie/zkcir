import { useState } from 'react';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Grid from '@mui/material/Grid';
import { styled } from '@mui/material/styles';
import { AnimatedTree } from 'react-tree-graph';
import ir_view from '../helpers/ir_view';
import styles from '../styles/tree.css';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import TabContext from '@mui/lab/TabContext';
import TabPanel from '@mui/lab/TabPanel';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import { TreeView } from '@mui/x-tree-view/TreeView';
import { TreeItem } from '@mui/x-tree-view/TreeItem';

const Item = styled(Paper)(({ theme }) => ({
  backgroundColor: theme.palette.mode === 'dark' ? '#1A2027' : '#fff',
  ...theme.typography,
  padding: theme.spacing(1),
  textAlign: 'center',
  color: theme.palette.text.secondary,
}));

function Dashboard() {
  const [tree, setTree] = useState(null);
  const [inputJson, setInputJson] = useState('');
  const [tab, setTab] = useState(0);
  const [expandedNodeIds, setExpandedNodeIds] = useState([]);

  const handleTabChange = (event, newValue) => {
    setTab(newValue);
  };

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

  const generateNodeIds = (node, nodeId = 0, ids = []) => {
    ids.push(nodeId.toString());
    if (Array.isArray(node.children)) {
      node.children.forEach((child, index) =>
        generateNodeIds(child, nodeId * 10 + index + 1, ids),
      );
    }
    return ids;
  };

  const handleJsonChange = (event) => {
    setInputJson(event.target.value);
  };

  const handleNodeToggle = (event, nodeIds) => {
    setExpandedNodeIds(nodeIds);
  };

  // Update handleSubmit to only set the tree and not expandedNodeIds
  const handleSubmit = () => {
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
      alert('Invalid JSON');
    }
  };

  return (
    <>
      <Box sx={{ flexGrow: 1, margin: '2%' }}>
        <Grid container spacing={2}>
          {/* Other grid items */}
          <Grid item xs={6}>
            <Item>
              <TextField
                label="JSON IR"
                multiline
                rows={27}
                value={inputJson}
                onChange={handleJsonChange}
                variant="outlined"
                fullWidth
                margin="normal"
              />
              <Button
                variant="contained"
                color="primary"
                onClick={handleSubmit}
              >
                Submit
              </Button>
            </Item>
          </Grid>
          <Grid item xs={6}>
            <Item>
              <TabContext value={tab.toString()}>
                <Tabs
                  value={tab.toString()}
                  onChange={handleTabChange}
                  aria-label="different IR views"
                  variant="fullWidth"
                >
                  <Tab label="AST" value="0" />
                  <Tab label="Tree" value="1" />
                </Tabs>
                <Item
                  style={{ justifyContent: 'center', alignItems: 'center' }}
                >
                  <TabPanel value="0">
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
                  </TabPanel>
                </Item>
                <Item style={{ justifyContent: 'left', alignItems: 'left' }}>
                  <TabPanel value="1">
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
                  </TabPanel>
                </Item>
              </TabContext>
            </Item>
          </Grid>
          {/* Other grid items */}
        </Grid>
      </Box>
    </>
  );
}

export default Dashboard;
