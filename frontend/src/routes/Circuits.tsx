import Upload from '../components/Upload';
import JSZip from 'jszip';
import { Button, Group } from '@mantine/core';
import { useForm } from 'react-hook-form';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { IconCode } from '@tabler/icons-react';

function Circuits() {
  const { register, handleSubmit, watch, setValue } = useForm<{
    files: FileList;
    entryIndex?: number;
  }>();
  const files = watch('files', new DataTransfer().files);
  const navigate = useNavigate();
  const user = useUser();

  useEffect(() => {
    if (!user.user) {
      //navigate('/auth/google');
    }
  }, [user.user, navigate]);

  const onSubmit = async () => {
    if (!files || files.length === 0) {
      alert('No files to process');
      return;
    }

    const zip = new JSZip();
    for (const file of files) {
      zip.file(file.webkitRelativePath || file.name, file);
    }

    try {
      const blob = await zip.generateAsync({ type: 'blob' });
      const formData = new FormData();
      formData.append('zip_file', blob, 'Circuit.zip');
      formData.append('cargo_args', 'cargo build --release');

      const response = await fetch('https://zkcir.chrisc.dev/v1/ir', {
        method: 'POST',
        body: formData,
        headers: {
          Authorization: `Bearer ${user.user?.token}`,
        },
      });

      if (!response.ok) {
        throw new Error('Network response was not ok');
      }

      const data = await response.json();
      console.log('Success:', data);
      alert('Compilation initiated successfully');
    } catch (error) {
      console.error('Error:', error);
      alert('Error initiating compilation');
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
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

export default Circuits;
