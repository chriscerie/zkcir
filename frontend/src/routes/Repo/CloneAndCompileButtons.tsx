import {
  Button,
  CopyButton,
  Flex,
  Group,
  Popover,
  Space,
  Text,
} from '@mantine/core';
import {
  IconCaretDownFilled,
  IconCheck,
  IconCode,
  IconCopy,
  IconHandFinger,
  IconMapPin,
  IconSourceCode,
} from '@tabler/icons-react';

export default function CloneAndCompileButtons({
  clone_url_ssh,
  entryPointPath,
}: {
  clone_url_ssh: string;
  entryPointPath?: string;
}) {
  return (
    <Popover radius="md" offset={0}>
      <Popover.Target>
        <div style={{ padding: '0.5rem' }}>
          <Button
            leftSection={<IconCode size="1.2rem" />}
            rightSection={<IconCaretDownFilled size="1.2rem" />}
            fullWidth
            size="xs"
          >
            Code
          </Button>
        </div>
      </Popover.Target>
      <Popover.Dropdown>
        <Flex direction="column">
          <Group gap="0.3rem">
            <IconSourceCode size="1.1rem" />
            <Text fw={600}>Clone with SSH</Text>
          </Group>
          <Space h="xs" />
          <CopyButton value={clone_url_ssh} timeout={2000}>
            {({ copied, copy }) => (
              <Button
                radius="md"
                variant="default"
                style={{
                  padding: '0.3rem 0 0.3rem 0.8rem',
                  height: 'auto',
                }}
                onClick={copy}
                justify="space-between"
                rightSection={
                  copied ? (
                    <IconCheck
                      size="1.5rem"
                      color={'teal'}
                      style={{ marginRight: '1rem' }}
                    />
                  ) : (
                    <IconCopy
                      size="1.5rem"
                      color={'gray'}
                      style={{ marginRight: '1rem' }}
                    />
                  )
                }
              >
                <Group justify="space-between">
                  <Group>
                    <div style={{ textAlign: 'left' }}>
                      <Text size="xs">
                        ssh://git-codecommit.us-east-1.amazonaws.com/ . . .
                      </Text>
                    </div>
                  </Group>
                </Group>
              </Button>
            )}
          </CopyButton>

          <Space h="xs" />

          <Group gap="0.3rem">
            <IconMapPin size="1.1rem" />
            <Text fw={600}>Compile</Text>
          </Group>

          <Space h="xs" />

          <Button
            radius="md"
            variant="default"
            style={{
              padding: '0.3rem 0 0.3rem 0.8rem',
              height: 'auto',
            }}
            justify="space-between"
            rightSection={
              <IconHandFinger
                size="1.5rem"
                color={'gray'}
                style={{ marginRight: '1rem' }}
              />
            }
          >
            <Group justify="space-between">
              <Group>
                <div style={{ textAlign: 'left' }}>
                  <Text size="xs">{entryPointPath || 'none selected'}</Text>
                </div>
              </Group>
            </Group>
          </Button>
        </Flex>
      </Popover.Dropdown>
    </Popover>
  );
}
