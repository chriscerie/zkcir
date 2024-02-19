import {
  AppShellMain,
  AppShellNavbar,
  Loader,
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
import {
  GetIrResponse,
  GetIrSourceResponse,
  GetRepoMetadataResponse,
} from '../../types';
import FileNode, { IFileNode } from './FileNode';
import classes from './index.module.css';
import IrEditor from './IrEditor';
import { SelectedSource } from './SelectedSource';
import CodeEditor from './CodeEditor';
import CloneAndCompileButtons from './CloneAndCompileButtons';

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

  const [entryPointPath, setEntryPointPath] = useState<string | undefined>();

  const { repo } = useParams();

  const repoMetadataUrl = `https://zkcir.chrisc.dev/v1/repo/metadata/${user?.user?.sub}/${repo}`;

  const { data: metadata } = useQuery<GetRepoMetadataResponse>(
    repoMetadataUrl,
    async () => {
      const response = await fetch(repoMetadataUrl, {
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

  const getIrUrl = `https://zkcir.chrisc.dev/v1/ir/${user.user?.sub}/${repo}/${metadata?.latest_commit_id}`;

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
      enabled: !!metadata?.latest_commit_id,
      staleTime: Infinity,
      refetchInterval: (data) => (data?.status === 200 ? false : 3000),
    },
  );

  const [selectedSource, setSelectedSource] = useState<
    SelectedSource | undefined
  >();

  const getIrSourceUrl = `https://zkcir.chrisc.dev/v1/repo/source/${user.user?.sub}/${repo}`;

  const {
    data: irSource,
    error: irSourceError,
    isLoading: isSourceLoading,
  } = useQuery<
    {
      items: IFileNode;
      downloadUrl: string;
    },
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

      const downloadUrl = window.URL.createObjectURL(response.data);

      const zip = new JSZip();

      return {
        items: await zip.loadAsync(response.data).then(async (zipFile) => {
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
        }),
        downloadUrl: downloadUrl,
      };
    },
    {
      staleTime: Infinity,
    },
  );

  const [active, setActive] = useState(2);

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
            {metadata && (
              <CloneAndCompileButtons
                repo_name={repo || ''}
                commit_id={metadata.latest_commit_id}
                clone_url_ssh={metadata.clone_url_ssh}
                entryPointPath={entryPointPath}
                onDownloadZip={() => {
                  window.open(irSource?.downloadUrl, '_blank');
                }}
              />
            )}
            {irSource && (
              <TreeView
                aria-label="file system navigator"
                defaultCollapseIcon={<IconChevronDown />}
                defaultExpandIcon={<IconChevronRight />}
              >
                {Object.entries(irSource.items).map(([name, node]) => (
                  <FileNode
                    key={name}
                    name={name}
                    node={node}
                    path={name}
                    entryPointPath={entryPointPath}
                    onFileClick={(fileName, path, contents) => {
                      setSelectedSource({ fileName, path, source: contents });
                    }}
                  />
                ))}
              </TreeView>
            )}

            {isSourceLoading && (
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'center',
                  alignItems: 'center',
                  height: '80vh',
                }}
              >
                <Loader color="blue" type="dots" />
              </div>
            )}
          </Allotment.Pane>
          <CodeEditor
            selectedSource={selectedSource}
            isSelectedEntryPoint={
              entryPointPath ? entryPointPath == selectedSource?.path : false
            }
            toggleEntryPoint={() => {
              setEntryPointPath((prevPath) => {
                if (prevPath == selectedSource?.path) {
                  return undefined;
                }
                return selectedSource?.path;
              });
            }}
          />
          <IrEditor
            repo={repo || ''}
            commit_id={metadata?.latest_commit_id || ''}
            jsonStr={irResponse?.data.json}
            cirStr={irResponse?.data.cir}
            isLoading={isSourceLoading || isIrLoading}
          />
        </Allotment>
      </AppShellMain>
    </>
  );
}
