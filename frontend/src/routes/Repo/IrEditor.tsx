import { Tabs, Text, rem, useMantineColorScheme } from '@mantine/core';
import { IconSettings } from '@tabler/icons-react';
import { Editor, Monaco } from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { useEffect, useState, useRef, useCallback } from 'react';
import { init } from 'echarts';
import ir_view from '../../helpers/ir_view';
import TreeComponent from '../../components/TreeComponent';
import CompilationStatusPage from './CompilationStatusPage';
import { GetIrStatusResponse } from '../../types';
import { IPosition, editor } from 'monaco-editor';

const COMPILATION = 'compilation';
const JSON = 'json';
const CIR = 'cir';
const AST = 'ast';
const AST_TREE = 'ast_tree';

// TODO: This is fragile. This should be replaced with a proper parser and lint through AST
// `verify!(virtual_wire::public(index: 0, value: 42u64) < (2u64 ^ 6u64));` lints on virtual_wire::public(index: 0, value: 42u64)
function findPublicInVerify(
  text: string,
): { start: IPosition; end: IPosition } | null {
  const verifyRegex = /verify!\((\w+::public\([^)]+\))/g;
  const match = verifyRegex.exec(text);
  if (!match) {
    return null;
  }

  const publicInput = match[1];
  const startOffset = match.index + 'verify!('.length;
  const endOffset = startOffset + publicInput.length;

  const start = byteOffsetToPosition(text, startOffset);
  const end = byteOffsetToPosition(text, endOffset);

  return { start, end };
}

function byteOffsetToPosition(text: string, offset: number): IPosition {
  let line = 1;
  let column = 1;

  for (let i = 0; i < offset; i++) {
    if (text[i] === '\n') {
      line++;
      column = 1;
    } else {
      column++;
    }
  }

  return { lineNumber: line, column: column };
}

export default function IrEditor({
  jsonStr,
  cirStr,
  isLoading,
  status,
}: {
  jsonStr?: string;
  cirStr?: string;
  isLoading: boolean;
  status?: GetIrStatusResponse;
}) {
  const { colorScheme } = useMantineColorScheme();

  const editorRef = useRef<editor.IStandaloneCodeEditor>();
  const monacoRef = useRef<Monaco>();

  const [page, setPage] = useState<string | null>(jsonStr ? CIR : COMPILATION);

  interface TreeNode {
    name: string;
    children?: TreeNode[];
    collapsed?: boolean;
  }

  const [tree, setTree] = useState<TreeNode | null>(null);
  const chartRef = useRef<HTMLDivElement>(null);

  const hasLoaded = !!jsonStr && !!cirStr;

  useEffect(() => {
    if (hasLoaded) {
      const parsedTree = ir_view.generateTree(jsonStr);
      setTree(parsedTree);
    } else {
      setTree({ name: 'Circuit', children: [] });
    }
  }, [hasLoaded, jsonStr]);

  useEffect(() => {
    if (page !== AST_TREE || !chartRef.current || !tree) {
      return;
    }

    const myChart = init(chartRef.current);
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

  const onIrChange = useCallback(
    (value?: string) => {
      const editorModel = editorRef.current?.getModel();
      const monaco = monacoRef.current;

      if (editorModel && monaco) {
        monaco.editor.removeAllMarkers('ir');

        if (value && page == CIR) {
          const lintPosition = findPublicInVerify(value);

          if (lintPosition) {
            monaco.editor.setModelMarkers(editorModel, 'ir', [
              {
                startLineNumber: lintPosition.start.lineNumber,
                startColumn: lintPosition.start.column,
                endLineNumber: lintPosition.end.lineNumber,
                endColumn: lintPosition.end.column,
                message:
                  'Public input is exposed to verifier. Ensure this is intended.',
                severity: monaco.MarkerSeverity.Warning,
              },
            ]);
          }
        }
      }
    },
    [page],
  );

  // `onChange` doesn't fire when value is set programmatically, so we need to call `onIrChange` manually when
  // the page changes
  useEffect(() => {
    onIrChange(page == CIR ? cirStr : jsonStr);
  }, [cirStr, jsonStr, onIrChange, page]);

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
          <Text size="sm">{'zkcir_out > compilation status'}</Text>
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
        <CompilationStatusPage
          onGoToIr={() => setPage(CIR)}
          status={status}
          isLoading={isLoading}
        />
      )}

      {(page == JSON || page == CIR) && !isLoading && (
        <Editor
          // Custom language cir is not supported, but using rust provides some syntax highlighting
          language={page == JSON ? 'json' : 'rust'}
          value={page == JSON ? jsonStr : cirStr}
          options={{
            // For some reason diagnostics don't show up if readOnly is enabled
            readOnly: false,
            padding: { top: 20 },
          }}
          theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
          onMount={(editor, monaco) => {
            editorRef.current = editor;
            monacoRef.current = monaco;
            onIrChange(editor.getValue());
          }}
          onChange={(value) => {
            onIrChange(value);
          }}
        />
      )}

      {page == AST && !isLoading && <TreeComponent tree={tree} />}

      {page == AST_TREE && !isLoading && (
        <div
          ref={chartRef}
          style={{ height: '60%', width: '100%', margin: '5%' }}
        ></div>
      )}
    </>
  );
}
