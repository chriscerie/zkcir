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
  IconDownload,
  IconPlayerPlay,
  IconSourceCode,
} from '@tabler/icons-react';
import { useMutation } from 'react-query';
import { Link } from 'react-router-dom';
import { useUser } from '../../UserContext';

export default function CloneAndCompileButtons({
  repo_name,
  commit_id,
  clone_url_ssh,
  entryPointPath,
  onDownloadZip,
}: {
  repo_name: string;
  commit_id?: string;
  clone_url_ssh: string;
  entryPointPath?: string;
  onDownloadZip: () => void;
}) {
  const user = useUser();

  const parts = entryPointPath?.split('/');
  const lastPart = parts && parts[parts.length - 1];
  const entryPointFileWithoutExtension = lastPart
    ?.split('.')
    .slice(0, -1)
    .join('.');

  const compileMutation = useMutation(
    () => {
      if (!entryPointFileWithoutExtension) {
        throw new Error('Need entry point');
      }

      return fetch(
        `https://zkcir.chrisc.dev/v1/ir/${user.user?.sub}/${repo_name}/${commit_id}`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${user.user?.auth_token}`,
          },
          body: JSON.stringify({
            example_artifact: entryPointFileWithoutExtension,
          }),
        },
      );
    },
    {
      onSuccess: () => {
        close();
      },
    },
  );

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

          <Space h="0.1rem" />

          <Text c="dimmed" size="xs">
            You must have an SSH key registered.{' '}
            <Link
              to="/settings"
              style={{
                textDecoration: 'none',
              }}
            >
              See more
            </Link>
          </Text>

          <Space h="lg" />

          <Button
            radius="md"
            variant="default"
            style={{
              padding: '0.3rem 0 0.3rem 0.8rem',
              height: 'auto',
            }}
            justify="left"
            fullWidth
            leftSection={
              <IconDownload
                size="1.5rem"
                color={'gray'}
                style={{ marginRight: '1rem' }}
              />
            }
            onClick={onDownloadZip}
          >
            <Group justify="space-between">
              <Group>
                <div style={{ textAlign: 'left' }}>
                  <Text size="sm" fw={600}>
                    Download ZIP
                  </Text>
                </div>
              </Group>
            </Group>
          </Button>

          <Space h="xs" />

          <Button
            radius="md"
            variant="default"
            style={{
              padding: '0.3rem 0 0.3rem 0.8rem',
              height: 'auto',
            }}
            justify="left"
            fullWidth
            leftSection={
              <IconPlayerPlay
                size="1.5rem"
                color={'gray'}
                style={{ marginRight: '1rem' }}
              />
            }
            onClick={() => compileMutation.mutate()}
            loading={compileMutation.isLoading}
            disabled={!commit_id}
          >
            <Group justify="space-between">
              <Group>
                <div style={{ textAlign: 'left' }}>
                  <Text size="sm" fw={600}>
                    Compile
                  </Text>
                  <Text size="xs">
                    {entryPointPath || 'select entry point'}
                  </Text>
                </div>
              </Group>
            </Group>
          </Button>
        </Flex>
      </Popover.Dropdown>
    </Popover>
  );
}
