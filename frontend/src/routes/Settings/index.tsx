import {
  AppShellMain,
  Container,
  Space,
  Stack,
  Text,
  Title,
} from '@mantine/core';
import { useUser } from '../../UserContext';
import KeyButton from './KeyButton';
import NewKey from './NewKey';
import { useQuery } from 'react-query';
import { ListKeysResponse } from '../../types';

const getKeysUrl = 'https://zkcir.chrisc.dev/v1/ssh';

export default function Setting() {
  const user = useUser();

  const { data: keys } = useQuery<ListKeysResponse>(
    getKeysUrl,
    async () => {
      const response = await fetch(getKeysUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      });

      return response.json();
    },
    {
      enabled: !!user.user,
      staleTime: Infinity,
    },
  );

  return (
    <AppShellMain>
      <Container
        size={700}
        style={{
          marginTop: '2rem',
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <Title order={4}>SSH Keys</Title>
          <NewKey />
        </div>

        <Text>Generate SSH keys and add the public key here.</Text>

        <Space h="lg" />

        <Stack>
          {keys?.keys?.map((keyInfo) => (
            <KeyButton
              key={keyInfo.id}
              keyId={keyInfo.id}
              uploadedTime={keyInfo.upload_time}
            />
          ))}
        </Stack>
      </Container>
    </AppShellMain>
  );
}
