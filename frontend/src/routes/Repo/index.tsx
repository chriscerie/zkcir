import {
  AppShellMain,
  AppShellNavbar,
  Stack,
  Tooltip,
  UnstyledButton,
  rem,
} from '@mantine/core';
import { useQuery } from 'react-query';
import axios, { AxiosError, AxiosResponse } from 'axios';
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
import { Allotment } from 'allotment';
import 'allotment/dist/style.css';
import { TreeView } from '@mui/x-tree-view';
import JSZip from 'jszip';
import { useUser } from '../../UserContext';
import NotFound from '../NotFound';
import {
  GetIrResponse,
  GetIrSourceResponse,
  GetIrVersionsResponse,
} from '../../types';
import FileNode, { IFileNode } from './FileNode';
import classes from './index.module.css';
import IrEditor from './IrEditor';
import { SelectedSource } from './SelectedSource';
import CodeEditor from './CodeEditor';

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

  const { repo } = useParams();

  const getVersionsUrl = `https://zkcir.chrisc.dev/v1/ir/versions/${repo}`;

  const {
    data: versions,
    error: versionsError,
    isLoading: isVersionsLoading,
  } = useQuery<GetIrVersionsResponse, AxiosError>(
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

  const getIrUrl = `https://zkcir.chrisc.dev/v1/ir/${repo}/${versions?.versions[0]}`;

  const {
    data: irResponse,
    error: irError,
    isLoading: isIrLoading,
  } = useQuery<AxiosResponse<GetIrResponse>, AxiosError>(
    getIrUrl,
    async () => {
      const response = await axios.get<GetIrResponse>(getIrUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      });

      return response;
    },
    {
      enabled: !!versions?.versions,
      staleTime: Infinity,
      refetchInterval: (data) => (data?.status === 200 ? false : 3000),
    },
  );

  const [selectedSource, setSelectedSource] = useState<
    SelectedSource | undefined
  >();

  const getIrSourceUrl = `https://zkcir.chrisc.dev/v1/ir/source/${user.user?.sub}/${repo}/${versions?.versions[0]}`;

  const {
    data: irSource,
    error: irSourceError,
    isLoading: isSourceLoading,
  } = useQuery<IFileNode, AxiosError>(
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
                    key={name}
                    name={name}
                    node={node}
                    path={name}
                    onFileClick={(fileName, path, contents) => {
                      setSelectedSource({ fileName, path, source: contents });
                    }}
                  />
                ))}
            </TreeView>
          </Allotment.Pane>
          <CodeEditor selectedSource={selectedSource} />
          <IrEditor
            jsonStr={irResponse?.data.json}
            cirStr={irResponse?.data.cir}
            isLoading={isVersionsLoading || isSourceLoading || isIrLoading}
          />
        </Allotment>
      </AppShellMain>
    </>
  );
}
