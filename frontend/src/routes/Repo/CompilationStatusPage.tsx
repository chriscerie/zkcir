import { Button, Space, Text, Timeline } from '@mantine/core';
import { IconGitBranch } from '@tabler/icons-react';
import 'allotment/dist/style.css';
import { AxiosError } from 'axios';
import { useQuery } from 'react-query';
import { useUser } from '../../UserContext';
import { GetIrStatusResponse } from '../../types';

const statusToIndex: { [key in GetIrStatusResponse]: number } = {
  [GetIrStatusResponse.NotStarted]: -1,
  [GetIrStatusResponse.CloningRepository]: 1,
  [GetIrStatusResponse.Compiling]: 2,
  [GetIrStatusResponse.Completed]: 3,
};

export default function IrEditor({
  repo,
  commit_id,
  onGoToIr,
  hasIrs,
}: {
  repo: string;
  commit_id: string;
  onGoToIr: () => void;
  hasIrs: boolean;
}) {
  const user = useUser();

  const getIrSourceUrl = `https://zkcir.chrisc.dev/v1/ir/status/${user.user?.sub}/${repo}/${commit_id}`;

  const { data: irStatus, isLoading: irIrStatusLoading } = useQuery<
    GetIrStatusResponse,
    AxiosError
  >(getIrSourceUrl, async () => {
    const response = await fetch(getIrSourceUrl, {
      headers: {
        Authorization: `Bearer ${user.user?.auth_token}`,
      },
    });

    if (!response.ok) {
      if (response.status == 500) {
        return GetIrStatusResponse.NotStarted;
      }
      throw new Error('Error encountered');
    }

    const responseText = await response.text();

    return responseText as GetIrStatusResponse;
  });

  // Status endpoint also returns the result, but if if fetching ir wins race condition, sync the statuses
  const syncedIrStatus = hasIrs ? GetIrStatusResponse.Completed : irStatus;

  if (irIrStatusLoading) {
    return <div>Loading...</div>;
  }

  const index = irStatus ? statusToIndex[irStatus] : 0;

  return (
    <div
      style={{
        padding: '2rem 2rem',
      }}
    >
      <Timeline active={index} bulletSize={24} lineWidth={2}>
        <Timeline.Item bullet={<IconGitBranch size={12} />} title="Initiated">
          <Text c="dimmed" size="sm">
            Started compilation
          </Text>
          <Text size="xs" mt={4}>
            2 hours ago
          </Text>
        </Timeline.Item>

        <Timeline.Item
          title="Cloning repository"
          bullet={<IconGitBranch size={12} />}
          lineVariant="dashed"
        >
          <Text c="dimmed" size="sm">
            Cloning repository
          </Text>
        </Timeline.Item>

        <Timeline.Item
          title="Compiling"
          bullet={<IconGitBranch size={12} />}
          lineVariant="dashed"
        >
          <Text c="dimmed" size="sm">
            Compiling to intermediate representation
          </Text>
        </Timeline.Item>

        <Timeline.Item
          title="Intermediate Representation"
          bullet={<IconGitBranch size={12} />}
        >
          <Text c="dimmed" size="sm">
            Finished compiling
          </Text>

          <Space h="lg" />

          {syncedIrStatus == GetIrStatusResponse.Completed && (
            <Button variant="outline" onClick={() => onGoToIr()}>
              Go to IR
            </Button>
          )}
        </Timeline.Item>
      </Timeline>
    </div>
  );
}
