import {
  AppShellMain,
  AppShellNavbar,
  Stack,
  Tooltip,
  UnstyledButton,
  rem,
  useMantineColorScheme,
} from '@mantine/core';
import { useQuery } from 'react-query';
import axios, { AxiosError } from 'axios';
import { useParams } from 'react-router-dom';
import {
  IconChevronDown,
  IconChevronRight,
  IconFiles,
  IconGitCommit,
  IconHome2,
  IconSettings,
} from '@tabler/icons-react';
import { useState } from 'react';
import { Editor } from '@monaco-editor/react';
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import { TreeView } from '@mui/x-tree-view';
import JSZip from 'jszip';
import { useUser } from '../../UserContext';
import NotFound from '../NotFound';
import {
  GetIrJsonResponse,
  GetIrSourceResponse,
  GetIrVersionsResponse,
} from '../../types';
import FileNode, { IFileNode } from './FileNode';
import classes from './index.module.css';

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
      staleTime: Infinity,
    },
  );

  const [selectedSource, setSelectedSource] = useState<{
    path: string;
    source: string;
  } | null>();

  const getIrSourceUrl = `https://zkcir.chrisc.dev/v1/ir/source/${user.user?.sub}/${repo}/${versions?.versions[0]}`;

  const { data: irSource, error: irSourceError } = useQuery<
    IFileNode,
    AxiosError
  >(
    getIrSourceUrl,
    async () => {
      const response = await axios.get<GetIrSourceResponse>(getIrSourceUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
        responseType: 'blob',
      });

      const zip = new JSZip();

      return await zip.loadAsync(response.data).then(async (zipFile) => {
        const items: IFileNode = {};

        await Promise.all(
          Object.keys(zipFile.files).map(async (relativePath) => {
            const file = zipFile.files[relativePath];
            const pathParts = relativePath.split('/');

            let current: IFileNode = items;

            for (let i = 0; i < pathParts.length; i++) {
              const isFile = i === pathParts.length - 1;
              const part = pathParts[i];
              if (!current[part]) {
                if (isFile) {
                  current[part] = await file.async('text');
                } else {
                  current[part] = {};
                }
              }
              if (!isFile) {
                current = current[part] as IFileNode;
              }
            }
          }),
        );

        return items;
      });
    },
    {
      enabled: !!versions?.versions,
      staleTime: Infinity,
    },
  );

  // Backend returns string of string
  const jsonStr = irJson?.ir
    .slice(1, -1)
    .replace(/\\n/g, '\n')
    .replace(/\\"/g, '"');

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

      <AppShellMain style={{ height: '100vh' }}>
        <Allotment defaultSizes={[0.9, 2, 2]}>
          <Allotment.Pane>
            <TreeView
              aria-label="file system navigator"
              defaultCollapseIcon={<IconChevronDown />}
              defaultExpandIcon={<IconChevronRight />}
            >
              {irSource &&
                Object.entries(irSource).map(([name, node]) => (
                  <FileNode
                    key="name"
                    name={name}
                    node={node}
                    path={name}
                    onFileClick={(path, contents) => {
                      setSelectedSource({ path, source: contents });
                    }}
                  />
                ))}
            </TreeView>
          </Allotment.Pane>
          <Allotment.Pane>
            <Editor
              language="rust"
              value={selectedSource?.source || undefined}
              theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
            />
          </Allotment.Pane>
          <Allotment.Pane>
            <Editor
              language="json"
              value={jsonStr}
              options={{
                readOnly: true,
              }}
              theme={colorScheme === 'dark' ? 'vs-dark' : 'light'}
            />
          </Allotment.Pane>
        </Allotment>
      </AppShellMain>
    </>
  );
}
