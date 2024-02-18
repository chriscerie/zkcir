import { Badge, Button, CopyButton, Group, Text } from '@mantine/core';
import { useUser } from '../../UserContext';
import { IconCheck, IconCopy, IconKey, IconTrash } from '@tabler/icons-react';
import { useState } from 'react';

export default function KeyButton({
  keyId,
  uploadedTime,
}: {
  keyId: string;
  uploadedTime: string;
}) {
  const user = useUser();
  const [isHovered, setIsHovered] = useState(false);

  return (
    <CopyButton value={keyId} timeout={2000}>
      {({ copied, copy }) => (
        <Button
          radius="md"
          variant="default"
          style={{
            padding: '1rem',
            height: 'auto',
          }}
          key={keyId}
          onClick={copy}
          justify="space-between"
          rightSection={
            <>
              {(isHovered || copied) &&
                (copied ? (
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
                ))}
              <Button
                variant="outline"
                color="red"
                size="xs"
                leftSection={<IconTrash size="1rem" />}
                onClick={(event) => {
                  event.stopPropagation();
                }}
                onMouseEnter={() => setIsHovered(false)}
                onMouseLeave={() => setIsHovered(true)}
              >
                Delete
              </Button>
            </>
          }
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
        >
          <Group justify="space-between">
            <Group>
              <IconKey size={'2.5rem'} />
              <div style={{ textAlign: 'left' }}>
                <Text size="xs" fw={600}>
                  {keyId}
                  <Badge
                    variant="outline"
                    size="xs"
                    style={{ margin: '0 0 0.2rem 0.4rem' }}
                  >
                    SSH Key ID
                  </Badge>
                </Text>
                <Text size="xs" c="dimmed">
                  {uploadedTime}
                </Text>
              </div>
            </Group>
          </Group>
        </Button>
      )}
    </CopyButton>
  );
}
