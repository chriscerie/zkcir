import { Button, Loader, Space, Text, Timeline } from '@mantine/core';
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
  isLoading,
  onGoToIr,
  status,
}: {
  onGoToIr: () => void;
  status?: GetIrStatusResponse;
  isLoading: boolean;
}) {
  if (isLoading) {
    return (
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '80vh',
        }}
      >
        <Loader color="blue" type="dots" />
      </div>
    );
  }

  // Status won't exist if it's an empty repo
  const index = status ? statusToIndex[status] : -1;

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
        </Timeline.Item>

        <Timeline.Item
          title="Cloning repository"
          bullet={
            <div
              style={{ position: 'relative', width: '20px', height: '20px' }}
            >
              <IconGitBranch
                size={12}
                style={{
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transform: 'translate(-50%, -50%)',
                }}
              />
              {index == 1 && (
                <Loader
                  color="blue"
                  style={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                  }}
                />
              )}
            </div>
          }
          lineVariant="dashed"
        >
          <Text c="dimmed" size="sm">
            Cloning repository
          </Text>
        </Timeline.Item>

        <Timeline.Item
          title="Compiling"
          bullet={
            <div
              style={{ position: 'relative', width: '20px', height: '20px' }}
            >
              <IconGitBranch
                size={12}
                style={{
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transform: 'translate(-50%, -50%)',
                }}
              />
              {index == 2 && (
                <Loader
                  color="blue"
                  style={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                  }}
                />
              )}
            </div>
          }
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
