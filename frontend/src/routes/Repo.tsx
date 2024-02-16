import {
  AppShellMain,
  AppShellNavbar,
  Stack,
  Tabs,
  Tooltip,
  UnstyledButton,
  rem,
  useMantineColorScheme,
} from '@mantine/core';
import { useQuery } from 'react-query';
import type { GetIrJsonResponse, GetIrVersionsResponse } from '../types';
import axios, { AxiosError } from 'axios';
import { useParams } from 'react-router-dom';
import { useUser } from '../UserContext';
import NotFound from './NotFound';
import {
  IconFiles,
  IconGitCommit,
  IconHome2,
  IconSettings,
} from '@tabler/icons-react';
import classes from './Repo.module.css';
import { useState } from 'react';
import { Editor } from '@monaco-editor/react';

const mockdata = [
  { icon: IconFiles, label: 'File' },
  { icon: IconGitCommit, label: 'Versions' },
];

function NavbarLink({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: typeof IconHome2;
  label: string;
  active?: boolean;
  onClick?(): void;
}) {
  return (
    <Tooltip label={label} position="right" transitionProps={{ duration: 0 }}>
      <UnstyledButton
        onClick={onClick}
        className={classes.link}
        data-active={active || undefined}
      >
        <Icon style={{ width: rem(35), height: rem(35) }} stroke={1.2} />
      </UnstyledButton>
    </Tooltip>
  );
}

export default function Repo() {
  const user = useUser();

  const { colorScheme } = useMantineColorScheme();

  const { repo } = useParams();

  const getVersionsUrl = `https://zkcir.chrisc.dev/v1/ir/versions/${repo}`;

  const { data: versions, error: versionsError } = useQuery<
    GetIrVersionsResponse,
    AxiosError
  >(
    getVersionsUrl,
    async () => {
      const response = await axios.get<GetIrVersionsResponse>(getVersionsUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      });

      return response.data;
    },
    {
      enabled: !!user.user,
    },
  );

  const getIrJsonUrl = `https://zkcir.chrisc.dev/v1/ir/${repo}/${versions?.versions[0]}`;

  const { data: irJson, error: irJsonError } = useQuery<
    GetIrJsonResponse,
    AxiosError
  >(
    getIrJsonUrl,
    async () => {
      const response = await axios.get<GetIrJsonResponse>(getIrJsonUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      });

      return response.data;
    },
    {
      enabled: !!versions?.versions,
    },
  );

  console.log(versions, irJson);

  const jsonStr = irJson?.ir
    .slice(1, -1)
    .replace(/\\n/g, '\n')
    .replace(/\\"/g, '"');

  let jsonObj;
  if (jsonStr) {
    try {
      jsonObj = JSON.parse(jsonStr);
      console.log(jsonObj);
    } catch (e) {
      console.error('Error parsing JSON:', e);
    }
  }

  const [active, setActive] = useState(2);

  if (versionsError?.status === 404) {
    return <NotFound />;
  }

  return (
    <>
      <AppShellNavbar style={{ padding: '0.8rem' }}>
        <div className={classes.navbarMain}>
          <Stack justify="center" gap={15}>
            {mockdata.map((link, index) => (
              <NavbarLink
                {...link}
                key={link.label}
                active={index === active}
                onClick={() => setActive(index)}
              />
            ))}
          </Stack>
        </div>

        <Stack justify="center" gap={0}>
          <NavbarLink icon={IconSettings} label="Change account" />
        </Stack>
      </AppShellNavbar>

      <AppShellMain>
        <Editor
          height="90vh"
          language="json"
          value={jsonStr}
          options={{
            readOnly: true,
          }}
          theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
        />
      </AppShellMain>

      <Tabs defaultValue="gallery" orientation="vertical" placement="left">
        <Tabs.List>
          <Tabs.Tab value="gallery">Gallery</Tabs.Tab>
          <Tabs.Tab value="messages">Messages</Tabs.Tab>
          <Tabs.Tab value="settings">Settings</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="gallery">Gallery tab content</Tabs.Panel>
        <Tabs.Panel value="messages">Messages tab content</Tabs.Panel>
        <Tabs.Panel value="settings">Settings tab content</Tabs.Panel>
      </Tabs>
    </>
  );
}
