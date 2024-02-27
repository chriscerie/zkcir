import { Badge, Flex, Paper, Title } from '@mantine/core';
import { IconBook2 } from '@tabler/icons-react';
import { Link } from 'react-router-dom';
import './RepoStackCard.scss';

export default function RepoStackCard({
  name,
  description,
  ownerSub,
}: {
  name: string;
  description: string;
  ownerSub: string;
}) {
  return (
    <Paper
      shadow="xs"
      radius="lg"
      withBorder
      p="xl"
      styles={{
        root: {
          padding: '1.5rem',
        },
      }}
    >
      <Flex align="center" gap={'xs'}>
        <IconBook2 size={24} />
        <Link to={`/${ownerSub}/${name}`} className="title-link">
          <Title order={5}>{name}</Title>
        </Link>
        <Badge variant="outline">Private</Badge>
      </Flex>

      <p>{description}</p>
    </Paper>
  );
}
