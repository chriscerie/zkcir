import { Tabs, Text, rem, useMantineColorScheme } from '@mantine/core';
import { IconSettings } from '@tabler/icons-react';
import { Editor } from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { SelectedSource } from './SelectedSource';
import { getLanguageInfo } from './Languages';

export default function IrEditor({
  selectedSource,
}: {
  selectedSource?: SelectedSource;
}) {
  const { colorScheme } = useMantineColorScheme();

  console.log(selectedSource?.path);

  return (
    <>
      {selectedSource ? (
        <>
          <Tabs variant="outline" value={selectedSource.path}>
            <Tabs.List>
              <Tabs.Tab
                value={selectedSource.path}
                leftSection={
                  <IconSettings style={{ width: rem(12), height: rem(12) }} />
                }
              >
                {selectedSource.fileName}
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel
              value={selectedSource.path}
              style={{
                padding: '0.2rem 1rem',
              }}
            >
              <Text size="sm">
                {selectedSource?.path.split('/').join(' > ')}
              </Text>
            </Tabs.Panel>
          </Tabs>
          <Editor
            language={
              getLanguageInfo(selectedSource.fileName.split('.').pop() || 'txt')
                .monacoLanguage
            }
            value={selectedSource.source}
            theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
            options={{
              padding: { top: '20rem' },
            }}
          />
        </>
      ) : (
        <div />
      )}
    </>
  );
}
