import { Dropzone } from '@mantine/dropzone';
import {
  Button,
  CloseButton,
  Fieldset,
  Group,
  Text,
  TextInput,
} from '@mantine/core';
import {
  IconBrandRust,
  IconCloudUpload,
  IconFile,
  IconJson,
  IconToml,
  IconUpload,
  IconX,
} from '@tabler/icons-react';
import { Control, Controller, UseFormRegister } from 'react-hook-form';
import { minimatch } from 'minimatch';

export type FormValues = {
  files: FileList;
  entryIndex?: number;
  repoName?: string;
};

export default function Upload({
  files,
  addFiles,
  onFileRemove,
  register,
  entryIndex,
  setEntryIndex,
  control,
}: {
  files: FileList;
  addFiles: (newFiles: File[]) => void;
  onFileRemove: (index: number) => void;
  register: UseFormRegister<FormValues>;
  entryIndex?: number;
  setEntryIndex: (index: number) => void;
  control: Control<FormValues, unknown, FormValues>;
}) {
  const { ref: dropzoneRef } = register('files');

  const hasCargoTomlInRoot = Array.from(files).some((file) =>
    file.webkitRelativePath.startsWith('Cargo.toml'),
  );

  const hasCargoToml = Array.from(files).some((file) =>
    file.webkitRelativePath.includes('Cargo.toml'),
  );

  return (
    <>
      <Dropzone
        onDrop={(files) => addFiles(files)}
        maxSize={30 * 1024 ** 2}
        style={{ marginTop: '2rem', padding: '3rem' }}
        radius="md"
        ref={dropzoneRef}
      >
        <Group justify="center" style={{ pointerEvents: 'none' }}>
          <Dropzone.Accept>
            <IconUpload
              style={{
                width: 'rem(52)',
                height: 'rem(52)',
                color: 'var(--mantine-color-blue-6)',
              }}
              stroke={1.5}
            />
          </Dropzone.Accept>
          <Dropzone.Reject>
            <IconX
              style={{
                width: 'rem(52)',
                height: 'rem(52)',
                color: 'var(--mantine-color-red-6)',
              }}
              stroke={1.5}
            />
          </Dropzone.Reject>
          <Dropzone.Idle>
            <IconCloudUpload
              style={{ width: '3rem', height: '3rem' }}
              stroke={1.5}
            />
          </Dropzone.Idle>
        </Group>
        <Text ta="center" fw={700} fz="lg" mt="xl">
          <Dropzone.Accept>Drop files here</Dropzone.Accept>
          <Dropzone.Reject>File must be less than 50mb</Dropzone.Reject>
          <Dropzone.Idle>Drag and Drop</Dropzone.Idle>
        </Text>
        <Text ta="center" fz="sm" mt="xs" c="dimmed">
          Must include cargo.toml
        </Text>
      </Dropzone>

      {files[0] && (
        <Button.Group
          orientation="vertical"
          p="0"
          style={{ marginTop: '1rem' }}
        >
          {Array.from(files).map((file, index) => {
            const pathParts = (file.webkitRelativePath || file.name).split('/');
            const pathWithoutTopFolder = pathParts.slice(1).join('/');
            const fileExtension = file.name.split('.').pop();

            // Ex: `/Cargo.toml`, `/src/main.rs`
            const normalizedPath =
              '/' +
                (hasCargoToml && !hasCargoTomlInRoot
                  ? pathWithoutTopFolder
                  : file.webkitRelativePath) || file.name;

            const isValidEntryPoint = minimatch(normalizedPath, '/examples/*');

            let Icon = IconFile;
            switch (fileExtension) {
              case 'rs':
                Icon = IconBrandRust;
                break;
              case 'toml':
                Icon = IconToml;
                break;
              case 'json':
                Icon = IconJson;
                break;
            }

            return (
              <Button
                key={index}
                fullWidth
                variant={entryIndex == index ? 'gradient' : 'default'}
                gradient={
                  entryIndex == index
                    ? { from: 'blue', to: 'teal', deg: 76 }
                    : undefined
                }
                radius="md"
                styles={{
                  label: {
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    width: '100%',
                  },
                  root: !isValidEntryPoint
                    ? {
                        backgroundColor: 'initial',
                        cursor: 'default',
                      }
                    : entryIndex !== index
                      ? {
                          color: 'teal',
                        }
                      : undefined,
                }}
                onClick={() => {
                  if (isValidEntryPoint) {
                    setEntryIndex(index);
                  }
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center' }}>
                  <Icon size={18} style={{ marginRight: '0.5rem' }} />
                  <Text size="md">
                    {'/' +
                      (hasCargoToml && !hasCargoTomlInRoot
                        ? pathWithoutTopFolder
                        : file.webkitRelativePath) || file.name}
                  </Text>
                </div>
                <CloseButton
                  size="md"
                  onClick={(e) => {
                    e.stopPropagation();
                    onFileRemove(index);
                  }}
                />
              </Button>
            );
          })}
        </Button.Group>
      )}

      <Fieldset
        legend="Generate Circuit Intermediate Representation"
        style={{ marginTop: '1rem' }}
        radius="md"
      >
        <Controller
          render={({ field }) => (
            <TextInput
              {...field}
              label="Repository name"
              required
              placeholder="My repository"
            />
          )}
          defaultValue=""
          name="repoName"
          control={control}
          rules={{
            required: true,
            pattern: {
              value: /^[a-z-]+$/,
              message:
                'Repository name can only contain lowercase letters and hyphens',
            },
          }}
        />

        <TextInput
          disabled
          label="Entry point"
          placeholder={
            entryIndex
              ? files[entryIndex].name
              : 'Select the entry point from the files list'
          }
        />
      </Fieldset>
    </>
  );
}
