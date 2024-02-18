import { Button, CopyButton, Group, Popover, Text } from '@mantine/core';
import { IconCheck, IconCopy } from '@tabler/icons-react';

export default function CloneAndCompileButtons({
  clone_url_ssh,
}: {
  clone_url_ssh: string;
}) {
  return (
    <Popover radius="md">
      <Popover.Target>
        <Button>Clone</Button>
      </Popover.Target>
      <Popover.Dropdown bg="var(--mantine-color-body)">
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
                    <Text size="xs">{clone_url_ssh}</Text>
                  </div>
                </Group>
              </Group>
            </Button>
          )}
        </CopyButton>
      </Popover.Dropdown>
    </Popover>
  );
}
