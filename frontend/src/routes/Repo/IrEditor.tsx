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
import { useEffect, useState } from 'react';

const COMPILATION = 'compilation';
const JSON = 'json';
const CIR = 'cir';

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

  const hasLoaded = !!jsonStr && !!cirStr;

  const [activeIndex, setActiveIndex] = useState(hasLoaded ? 3 : 2);

  useEffect(() => {
    if (hasLoaded) {
      setActiveIndex(3);
    } else {
      const interval = setInterval(() => {
        setActiveIndex((prevActive) => (prevActive === 2 ? 3 : 2));
      }, 1000);

      return () => {
        clearInterval(interval);
      };
    }
  }, [hasLoaded]);

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
    </>
  );
}
