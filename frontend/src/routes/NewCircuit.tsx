import {
  AppShellMain,
  Button,
  Container,
  Fieldset,
  Group,
  TextInput,
  Textarea,
} from '@mantine/core';
import { Controller, SubmitHandler, useForm } from 'react-hook-form';
import { IconCode } from '@tabler/icons-react';
import { useUser } from '../UserContext';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { useMutation } from 'react-query';

type FormValues = {
  repoName?: string;
  repoDescription?: string;
};

function NewCircuit() {
  const user = useUser();

  const navigate = useNavigate();

  const { handleSubmit, control } = useForm<FormValues>();

  useEffect(() => {
    if (!user.user) {
      window.location.href = '/auth/google';
    }
  }, [user.user]);

  const newRepoMutation = useMutation(
    async (repoName: string) => {
      const res = await fetch('https://zkcir.chrisc.dev/v1/repo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
        body: JSON.stringify({ repo_name: repoName }),
      });

      if (!res.ok) {
        throw new Error(res.body?.toString());
      }

      return res.json();
    },
    {
      onError: (error) => {
        console.error('Error:', error);
        alert('Error creating new repository');
      },
      onSuccess: (_, repoName) => {
        navigate(`/${user.user?.sub}/${repoName}`);
      },
    },
  );

  const onSubmit: SubmitHandler<FormValues> = async (data) => {
    if (!data.repoName) {
      alert('Please enter a repository name');
      return;
    }

    newRepoMutation.mutate(data.repoName);
  };

  return (
    <AppShellMain>
      <Container size="lg">
        <form
          onSubmit={handleSubmit(onSubmit, (e) => {
            console.error('Compiling failed:', e);
          })}
        >
          <Fieldset
            legend="Create repository"
            style={{ marginTop: '1rem' }}
            radius="md"
            disabled={newRepoMutation.isLoading}
          >
            <Controller
              render={({ field }) => (
                <TextInput
                  {...field}
                  label="Name"
                  required
                  placeholder="my-repository"
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

            <Controller
              render={({ field }) => (
                <Textarea
                  {...field}
                  label="Description"
                  placeholder="My repository description"
                />
              )}
              defaultValue=""
              name="repoDescription"
              control={control}
            />
          </Fieldset>

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
              loading={newRepoMutation.isLoading}
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
      </Container>
    </AppShellMain>
  );
}

export default NewCircuit;
