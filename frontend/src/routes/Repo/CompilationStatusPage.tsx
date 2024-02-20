import { Button, Space, Text, Timeline } from '@mantine/core';
import { IconGitBranch } from '@tabler/icons-react';
import 'allotment/dist/style.css';
import { GetIrStatusResponse } from '../../types';

const statusToIndex: { [key in GetIrStatusResponse]: number } = {
  [GetIrStatusResponse.NotStarted]: -1,
  [GetIrStatusResponse.CloningRepository]: 1,
  [GetIrStatusResponse.Compiling]: 2,
  [GetIrStatusResponse.Completed]: 3,
};

export default function IrEditor({
  onGoToIr,
  status,
}: {
  onGoToIr: () => void;
  status?: GetIrStatusResponse;
}) {
  if (!status) {
    return <div>Loading...</div>;
  }

  const index = statusToIndex[status];

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

          {status == GetIrStatusResponse.Completed && (
            <Button variant="outline" onClick={() => onGoToIr()}>
              Go to IR
            </Button>
          )}
        </Timeline.Item>
      </Timeline>
    </div>
  );
}
