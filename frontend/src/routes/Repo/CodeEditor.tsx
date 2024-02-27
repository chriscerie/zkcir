import {
  Chip,
  Group,
  Tabs,
  Text,
  Tooltip,
  rem,
  useMantineColorScheme,
} from '@mantine/core';
import {
  IconMapPin,
  IconMapPinFilled,
  IconSettings,
} from '@tabler/icons-react';
import { Editor } from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { SelectedSource } from './SelectedSource';
import { getLanguageInfo } from './Languages';
import { isValidEntryPoint } from './EntryPoint';
import { IFileNode } from './FileNode';
import DragAndDrop from '../../components/DragAndDrop'


export default function IrEditor({
  files,
  loading,
  selectedSource,
  isSelectedEntryPoint,
  toggleEntryPoint,
}: {
  files?: IFileNode;
  loading: boolean;
  selectedSource?: SelectedSource;
  isSelectedEntryPoint: boolean;
  toggleEntryPoint: (entryPoint: boolean) => void;
}) {
  const { colorScheme } = useMantineColorScheme();

  function isEmptyFileNode(node: IFileNode | undefined): boolean {
    for (const key in node) {
      if (Object.prototype.hasOwnProperty.call(node, key)) {
        const value = node[key];
        if (typeof value === 'object' && value !== null && !isEmptyFileNode(value)) {
          return false;
        } else if (typeof value === 'string') {
          return false;
        }
      }
    }
    return true;
  }

  return (
    <>
      {isEmptyFileNode(files) && !loading ? (
        <DragAndDrop />
      ) : selectedSource ? (
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
              <Group>
                <Tooltip
                  label={
                    isValidEntryPoint(selectedSource.path)
                      ? 'Select as entry point'
                      : 'Not a recognized entry point'
                  }
                  refProp="rootRef"
                >
                  <Chip
                    size="xs"
                    variant="outline"
                    checked={isSelectedEntryPoint}
                    disabled={!isValidEntryPoint(selectedSource.path)}
                    onChange={toggleEntryPoint}
                  >
                    <Group gap="0.2rem">
                      {isSelectedEntryPoint ? (
                        <IconMapPinFilled size="0.9rem" />
                      ) : (
                        <IconMapPin size="0.9rem" />
                      )}
                      Entry point
                    </Group>
                  </Chip>
                </Tooltip>
                <Text size="sm">
                  {selectedSource?.path.split('/').join(' > ')}
                </Text>
              </Group>
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
