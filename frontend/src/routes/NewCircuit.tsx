import Upload, { FormValues } from '../components/Upload';
import JSZip from 'jszip';
import { Button, Group } from '@mantine/core';
import { SubmitHandler, useForm } from 'react-hook-form';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { IconCode } from '@tabler/icons-react';
import axios from 'axios';

function NewCircuit() {
  const { register, handleSubmit, watch, setValue, control } =
    useForm<FormValues>();
  const files = watch('files', new DataTransfer().files);
  const navigate = useNavigate();
  const user = useUser();

  useEffect(() => {
    if (!user.user) {
      //navigate('/auth/google');
    }
  }, [user.user, navigate]);

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

    if (!data.entryIndex) {
      alert('Please select an entry file');
      return;
    }

    if (!data.repoName) {
      alert('Please enter a repository name');
      return;
    }

    const token = localStorage.getItem('token');

    const nameWithoutExtension = data.files[data.entryIndex].name
      .split('.')
      .slice(0, -1)
      .join('.');

    try {
      const blob = await zip.generateAsync({ type: 'blob' });
      const formData = new FormData();
      formData.append('zip_file', blob, 'Circuit.zip');
      formData.append('example_artifact', nameWithoutExtension);
      formData.append('repo_name', data.repoName);

      axios
        .post<{
          repo_name: string;
          circuit_version: string;
        }>('https://zkcir.chrisc.dev/v1/ir', formData, {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        })
        .then((response) => {
          console.log('Success:', response.data);
        })
        .catch((error) => {
          console.error('Error:', error);
          alert('Error initiating compilation');
        });
    } catch (error) {
      console.error('Error:', error);
      alert('Error initiating compilation');
    }
  };

  return (
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
              !file.webkitRelativePath.includes('target/release'),
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
        >
          Compile
        </Button>

        <Button
          variant="transparent"
          color="pink"
          radius="md"
          style={{ padding: '0 0.3rem' }}
        >
          Cancel
        </Button>
      </Group>
    </form>
  );
}

export default NewCircuit;
