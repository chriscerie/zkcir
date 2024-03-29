import Upload, { FormValues } from './Upload';
import JSZip from 'jszip';
import { Button, Container, Group } from '@mantine/core';
import { SubmitHandler, useForm } from 'react-hook-form';
import { IconCode } from '@tabler/icons-react';
import axios from 'axios';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { useMutation } from 'react-query';

function DragAndDrop() {
  const user = useUser();

  const navigate = useNavigate();

  const { register, handleSubmit, watch, setValue, control } =
    useForm<FormValues>();
  const files = watch('files', new DataTransfer().files);

  useEffect(() => {
    if (!user.user) {
      window.location.href = '/auth/google';
    }
  }, [user.user]);

  const compileMutation = useMutation(
    (formData: FormData) =>
      axios.post<{
        repo_name: string;
        circuit_version: string;
      }>('https://zkcir.chrisc.dev/v1/ir', formData, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      }),
    {
      onSuccess: (data) => {
        navigate(`/${user.user?.sub}/${data.data.repo_name}`);
      },
      onError: (error) => {
        console.error('Error:', error);
        alert('Error initiating compilation');
      },
    },
  );

  const onSubmit: SubmitHandler<FormValues> = async (data) => {
    const zip = new JSZip();

    // If user drops a project folder, Cargo.toml would be in folder/Cargo.toml, so remove the top layer
    // If user drops the files directly, Cargo.toml would be at the top level
    const shouldNotRemoveTopLevelDirectory = Array.from(data.files).some(
      (file) => file.webkitRelativePath.startsWith('Cargo.toml'),
    );

    for (const file of data.files) {
      const normalizedPath = shouldNotRemoveTopLevelDirectory
        ? file.webkitRelativePath
        : file.webkitRelativePath.split('/').slice(1).join('/');
      zip.file(normalizedPath || file.name, file);
    }

    try {
      const blob = await zip.generateAsync({ type: 'blob' });
      const formData = new FormData();
      formData.append('zip_file', blob, 'Circuit.zip');
      compileMutation.mutate(formData);
    } catch (error) {
      console.error('Error:', error);
      alert('Error initiating compilation');
    }
  };

  return (
    <Container size="lg">
      <form
        onSubmit={handleSubmit(onSubmit, (e) => {
          console.error('Compiling failed:', e);
        })}
      >
        <Upload
          files={files}
          addFiles={(newFiles) => {
            const filteredFiles = newFiles.filter(
              (file) =>
                !file.webkitRelativePath.includes('target/debug') &&
                !file.webkitRelativePath.includes('target/release') &&
                !file.webkitRelativePath.includes('.git/'),
            );
            const dataTransfer = new DataTransfer();
            Array.from(files).forEach((file) => dataTransfer.items.add(file));
            filteredFiles.forEach((file) => dataTransfer.items.add(file));
            setValue('files', dataTransfer.files, { shouldDirty: true });
          }}
          onFileRemove={(index) => {
            const newFiles = Array.from(files).filter((_, i) => i !== index);
            const dataTransfer = new DataTransfer();
            newFiles.forEach((file) => dataTransfer.items.add(file));
            setValue('files', dataTransfer.files, { shouldDirty: true });

            if (index === watch('entryIndex')) {
              setValue('entryIndex', undefined);
            }
          }}
          register={register}
          entryIndex={watch('entryIndex')}
          setEntryIndex={(index) => setValue('entryIndex', index)}
          control={control}
          isLoading={compileMutation.isLoading}
        />
        <Group style={{ marginTop: '0.7rem' }}>
          <Button
            variant="filled"
            color="green"
            radius="md"
            type="submit"
            leftSection={<IconCode size={'1.2rem'} />}
            styles={{
              root: {
                color: 'white',
              },
            }}
            loading={compileMutation.isLoading}
          >
            Upload
          </Button>
        </Group>
      </form>
    </Container>
  );
}

export default DragAndDrop;
