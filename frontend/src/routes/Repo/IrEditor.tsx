import {
  Button,
  Space,
  Tabs,
  Text,
  Timeline,
  rem,
  useMantineColorScheme,
} from '@mantine/core';
import { IconGitBranch, IconSettings } from '@tabler/icons-react';
import { Editor } from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { useEffect, useState, useRef } from 'react';
import * as echarts from 'echarts';
import ir_view from '../../helpers/ir_view';

const COMPILATION = 'compilation';
const JSON = 'json';
const CIR = 'cir';
const AST = 'ast'
const AST_TREE = 'ast_tree'

export default function IrEditor({
  jsonStr,
  cirStr,
  isLoading,
}: {
  jsonStr?: string;
  cirStr?: string;
  isLoading: boolean;
}) {
  const { colorScheme } = useMantineColorScheme();

  const [page, setPage] = useState<string | null>(jsonStr ? CIR : COMPILATION);
  
  interface TreeNode {
    name: string;
    children?: TreeNode[];
    collapsed?: boolean;
  }

  const [tree, setTree] = useState<TreeNode | null>(null);
  const chartRef = useRef<HTMLDivElement>(null);

  const hasLoaded = !!jsonStr && !!cirStr;

  const [activeIndex, setActiveIndex] = useState(hasLoaded ? 3 : 2);

  useEffect(() => {
    if (hasLoaded) {
      console.log(jsonStr);
      const parsedTree = ir_view.generateTree(jsonStr); // Make sure this is compatible
      setTree(parsedTree);
      setActiveIndex(3);
    } else {
      setTree({ name: 'Circuit', children: [] });
      const interval = setInterval(() => {
        setActiveIndex((prevActive) => (prevActive === 2 ? 3 : 2));
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [hasLoaded, jsonStr]);

  useEffect(() => {
    console.log(page)
    console.log(!chartRef.current)
    console.log(tree)
    if (page !== AST_TREE || !chartRef.current || !tree) {
      console.log('hi')
      return;
      
    }
    console.log('hi')
    

    const myChart = echarts.init(chartRef.current);
    myChart.showLoading();


    myChart.hideLoading();
    myChart.setOption({
      tooltip: {
        trigger: 'item',
        triggerOn: 'mousemove',
      },
      series: [
        {
          type: 'tree',
          data: [tree],
          top: '1%',
          left: '7%',
          bottom: '1%',
          right: '20%',
          orient: 'vertical',
          symbolSize: 10,
          label: {
            position: 'left',
            verticalAlign: 'middle',
            align: 'right',
            fontSize: 9,
          },
          leaves: {
            label: {
              position: 'right',
              verticalAlign: 'middle',
              align: 'left',
            },
          },
          emphasis: {
            focus: 'descendant',
          },
          expandAndCollapse: true,
          animationDuration: 550,
          animationDurationUpdate: 750,
        },
      ],
    });

    return () => {
      myChart.dispose();
    };

  }, [tree, page]);
  

  return (
    <>
      <Tabs variant="outline" value={page} onChange={setPage}>
        <Tabs.List>
          <Tabs.Tab
            value={COMPILATION}
            leftSection={
              <IconSettings style={{ width: rem(12), height: rem(12) }} />
            }
          >
            compilation
          </Tabs.Tab>

          <Tabs.Tab
            value={CIR}
            leftSection={
              <IconSettings style={{ width: rem(12), height: rem(12) }} />
            }
            disabled={!jsonStr}
          >
            ir.cir
          </Tabs.Tab>

          <Tabs.Tab
            value={JSON}
            leftSection={
              <IconSettings style={{ width: rem(12), height: rem(12) }} />
            }
            disabled={!jsonStr}
          >
            ir.json
          </Tabs.Tab>

          <Tabs.Tab
            value={AST}
            leftSection={
              <IconSettings style={{ width: rem(12), height: rem(12) }} />
            }
            disabled={!jsonStr}
          >
            ir.ast
          </Tabs.Tab>
          <Tabs.Tab
            value={AST_TREE}
            leftSection={
              <IconSettings style={{ width: rem(12), height: rem(12) }} />
            }
            disabled={!jsonStr}
          >
            ir.ast_tree
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel
          value={COMPILATION}
          style={{
            padding: '0.2rem 1rem',
          }}
        >
          <Text size="sm">compilation progress</Text>
        </Tabs.Panel>

        <Tabs.Panel
          value={CIR}
          style={{
            padding: '0.2rem 1rem',
          }}
        >
          <Text size="sm">{'zkcir_out > ir.cir'}</Text>
        </Tabs.Panel>

        <Tabs.Panel
          value={JSON}
          style={{
            padding: '0.2rem 1rem',
          }}
        >
          <Text size="sm">{'zkcir_out > ir.json'}</Text>
        </Tabs.Panel>
      </Tabs>

      {page == COMPILATION && !isLoading && (
        <div
          style={{
            padding: '2rem 2rem',
          }}
        >
          <Timeline active={activeIndex} bulletSize={24} lineWidth={2}>
            <Timeline.Item
              bullet={<IconGitBranch size={12} />}
              title="Initiated"
            >
              <Text c="dimmed" size="sm">
                Started compilation
              </Text>
              <Text size="xs" mt={4}>
                2 hours ago
              </Text>
            </Timeline.Item>

            <Timeline.Item
              title="Patched dependencies"
              bullet={<IconGitBranch size={12} />}
              lineVariant="dashed"
            >
              <Text c="dimmed" size="sm">
                Found `plonky2`, patched dependencies
              </Text>
            </Timeline.Item>

            <Timeline.Item
              title="Compiling"
              bullet={<IconGitBranch size={12} />}
              lineVariant="dashed"
            >
              <Text c="dimmed" size="sm">
                Compiling to intermediate representation
              </Text>
            </Timeline.Item>

            <Timeline.Item
              title="Intermediate Representation"
              bullet={<IconGitBranch size={12} />}
            >
              <Text c="dimmed" size="sm">
                Finished compiling
              </Text>

              <Space h="lg" />

              {activeIndex == 3 && (
                <Button variant="outline" onClick={() => setPage(CIR)}>
                  Go to IR
                </Button>
              )}
            </Timeline.Item>
          </Timeline>
        </div>
      )}

      {(page == JSON || page == CIR) && !isLoading && (
        <Editor
          // Custom language cir is not supported, but using rust provides some syntax highlighting
          language={page == JSON ? 'json' : 'rust'}
          value={page == JSON ? jsonStr : cirStr}
          options={{
            readOnly: true,
            padding: { top: '20rem' },
          }}
          theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
        />
      )}

      {(page == AST) && !isLoading && (
        null
      )}

      {(page == AST_TREE) && !isLoading && (
        <div ref={chartRef} style={{ height: '60%', width: '100%', margin:'5%' }}></div>
      )}
    </>
  );
}
