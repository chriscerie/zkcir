import { SimpleGrid } from '@mantine/core';
import RepoStackCard from '../components/RepoStackCard';
import { useQuery } from 'react-query';
import type { ListIrsMetadataResponse } from '../types';
import axios from 'axios';

const data = [
  {
    name: 'Athena Weissnat',
    description: 'Little - Rippin',
  },
  {
    name: 'Deangelo Runolfsson',
    description: 'Greenfelder - Krajcik',
  },
  {
    name: 'Athena Weissnat',
    description: 'Little - Rippin',
  },
  {
    name: 'Deangelo Runolfsson',
    description: 'Greenfelder - Krajcik',
  },
];

const listIrsMetaDataUrl = 'https://zkcir.chrisc.dev/v1/ir/metadata/list';

export default function Home() {
  const token = localStorage.getItem('token');

  const { data: irs, isLoading } = useQuery(listIrsMetaDataUrl, async () => {
    const response = await axios.get<ListIrsMetadataResponse>(
      listIrsMetaDataUrl,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );
    return response.data;
  });

  console.log(irs, isLoading);

  return (
    <SimpleGrid
      cols={2}
      style={{
        marginTop: '1.5rem',
      }}
    >
      {data.map((item, index) => (
        <RepoStackCard
          name={item.name}
          description={item.description}
          key={index}
        />
      ))}
    </SimpleGrid>
  );
}
