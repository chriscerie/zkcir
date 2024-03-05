import { Badge, Flex, Paper, Space, Text, Title } from '@mantine/core';
import { IconBook2, IconCircleFilled } from '@tabler/icons-react';
import { Link } from 'react-router-dom';
import './RepoStackCard.scss';
import { TargetFramework } from '../types';
import { formatDistanceToNow } from 'date-fns';

export default function RepoStackCard({
  name,
  description,
  ownerSub,
  last_modified_date,
  framework,
}: {
  name: string;
  description: string;
  ownerSub: string;
  last_modified_date: number;
  framework: TargetFramework;
}) {
  const date = new Date(last_modified_date * 1000);
  const formattedDate = formatDistanceToNow(date, { addSuffix: true });

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

      <Space h="xs" />

      <Flex align="center" gap={'1.8rem'}>
        <Flex align="center" gap={'0.2rem'}>
          <IconCircleFilled size={16} style={{ color: 'DarkOrchid' }} />
          <Text size="sm">{framework}</Text>
        </Flex>
        <Text size="sm">Updated {formattedDate}</Text>
      </Flex>

      <p>{description}</p>
    </Paper>
  );
}
