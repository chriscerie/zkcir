import { Tabs, Text, rem, useMantineColorScheme } from '@mantine/core';
import { IconSettings } from '@tabler/icons-react';
import { Editor } from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { useState } from 'react';

export default function IrEditor({ jsonStr }: { jsonStr?: string }) {
  const { colorScheme } = useMantineColorScheme();

  const [page, setPage] = useState('json');

  return (
    jsonStr && (
      <>
        <Tabs variant="outline" value={page}>
          <Tabs.List>
            <Tabs.Tab
              value="json"
              leftSection={
                <IconSettings style={{ width: rem(12), height: rem(12) }} />
              }
            >
              output.json
            </Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel
            value="json"
            style={{
              padding: '0.2rem 1rem',
            }}
          >
            <Text size="sm">output.json</Text>
          </Tabs.Panel>
        </Tabs>

        <Editor
          language="json"
          value={jsonStr}
          options={{
            readOnly: true,
            padding: { top: '20rem' },
          }}
          theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
        />
      </>
    )
  );
}
