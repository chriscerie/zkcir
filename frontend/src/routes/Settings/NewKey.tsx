import { Button, Modal, Space, Textarea } from '@mantine/core';
import { useUser } from '../../UserContext';
import { useDisclosure } from '@mantine/hooks';
import { useMutation } from 'react-query';
import { Controller, useForm } from 'react-hook-form';

type FormValues = {
  key: string;
};

export default function NewKey() {
  const user = useUser();
  const [opened, { open, close }] = useDisclosure(false);
  const { handleSubmit, control } = useForm<FormValues>();

  const mutation = useMutation(
    (key: string) => {
      return fetch('https://zkcir.chrisc.dev/v1/ssh', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
        body: JSON.stringify({
          key,
        }),
      });
    },
    {
      onSuccess: () => {
        close();
      },
    },
  );

  return (
    <>
      <Modal opened={opened} onClose={close} title="New RSA SSH key" size="lg">
        <form onSubmit={handleSubmit((data) => mutation.mutate(data.key))}>
          <Controller
            render={({ field }) => (
              <Textarea
                {...field}
                label="Paste the contents of the public key into the following field."
                placeholder="ssh-rsa ..."
                resize="vertical"
                autosize
                minRows={15}
                maxRows={30}
                disabled={mutation.isLoading}
              />
            )}
            defaultValue=""
            name="key"
            control={control}
          />

          <Space h="sm" />

          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <Button
              size="xs"
              color="green"
              radius="md"
              styles={{
                root: {
                  color: 'white',
                },
              }}
              type="submit"
              loading={mutation.isLoading}
            >
              Add SSH key
            </Button>
          </div>
        </form>
      </Modal>

      <Button
        size="xs"
        color="green"
        radius="md"
        styles={{
          root: {
            color: 'white',
          },
        }}
        onClick={open}
      >
        New SSH key
      </Button>
    </>
  );
}
