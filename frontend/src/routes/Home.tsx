import {
  AppShellMain,
  Button,
  Container,
  SimpleGrid,
  Skeleton,
  Space,
  Text,
} from '@mantine/core';
import RepoStackCard from '../components/RepoStackCard';
import { useQuery } from 'react-query';
import type { ListReposResponse } from '../types';
import axios from 'axios';
import { useEffect, useState } from 'react';
import classes from './Home.module.css';
import { Link } from 'react-router-dom';
import { useUser } from '../UserContext';

const listIrsMetaDataUrl = 'https://zkcir.chrisc.dev/v1/repos';

export default function Home() {
  const user = useUser();

  const { data: irs, isLoading } = useQuery(
    listIrsMetaDataUrl,
    async () => {
      const response = await axios.get<ListReposResponse>(listIrsMetaDataUrl, {
        headers: {
          Authorization: `Bearer ${user.user?.auth_token}`,
        },
      });
      return response.data;
    },
    {
      enabled: !!user.user,
    },
  );

  const [showLoading, setShowLoading] = useState(false);

  useEffect(() => {
    let timer: NodeJS.Timeout;
    if (isLoading) {
      timer = setTimeout(() => setShowLoading(true), 500);
    } else {
      setShowLoading(false);
    }
    return () => {
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [isLoading]);

  return (
    <AppShellMain>
      <Container
        size={900}
        className={classes.inner}
        style={{
          marginTop: '2rem',
        }}
      >
        {!isLoading &&
          (!user.user || !irs?.repos || irs?.repos.length == 0) && (
            <>
              <Space h="xl" />
              <h1 className={classes.title}>
                A{' '}
                <Text
                  component="span"
                  variant="gradient"
                  gradient={{ from: 'blue', to: 'cyan' }}
                  inherit
                >
                  universal
                </Text>{' '}
                platform for developing, compiling, and auditing ZK circuits
              </h1>

              <Text className={classes.description} c="dimmed">
                Generate intermediate representations for zero knowledge
                circuits to help analyze and find security flaws over a
                framework-agnostic environment.
              </Text>

              <Link to="/new-circuit">
                <Button
                  size="xl"
                  className={classes.control}
                  variant="gradient"
                  gradient={{ from: 'blue', to: 'cyan' }}
                  style={{
                    marginTop: '2rem',
                  }}
                >
                  Create new circuit
                </Button>
              </Link>
            </>
          )}

        <SimpleGrid cols={2}>
          {showLoading ? (
            <>
              <Skeleton height={8} mt={6} width="100%" radius="xl" />
              <Skeleton height={8} mt={6} width="100%" radius="xl" />
              <Skeleton height={8} mt={6} width="100%" radius="xl" />
              <Skeleton height={8} mt={6} width="100%" radius="xl" />
              <Skeleton height={8} mt={6} width="100%" radius="xl" />
              <Skeleton height={8} mt={6} width="70%" radius="xl" />
            </>
          ) : (
            (irs?.repos || [])
              .sort((a, b) => b.last_modified_date - a.last_modified_date)
              .map((item, index) => (
                <RepoStackCard
                  name={item.name}
                  description={''}
                  key={index}
                  ownerSub={user.user?.sub || ''}
                  last_modified_date={item.last_modified_date}
                  framework={item.framework}
                />
              ))
          )}
        </SimpleGrid>
      </Container>
    </AppShellMain>
  );
}
