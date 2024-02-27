import 'allotment/dist/style.css';
import { TreeItem } from '@mui/x-tree-view';
import { getLanguageInfo } from './Languages';
import {
  IconBrandPocket,
  IconMapPin,
  IconMapPinFilled,
} from '@tabler/icons-react';
import { isValidEntryPoint } from './EntryPoint';

export interface IFileNode {
  // [file name]: source | FileNode
  [key: string]: string | IFileNode;
}

export default function FileNode({
  name,
  node,
  path,
  entryPointPath,
  onFileClick,
}: {
  name: string;
  node: string | IFileNode;
  // Not including leading slash. If this node is a file, this includes the file name
  path: string;
  entryPointPath?: string;
  onFileClick: (fileName: string, path: string, contents: string) => void;
}) {
  const IconComponent = getLanguageInfo(
    name.split('.').pop() || 'txt',
  ).iconComponent;

  return typeof node == 'string' ? (
    <TreeItem
      nodeId={path}
      label={
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            width: '100%',
          }}
        >
          <span
            style={{
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {name}
          </span>
          {isValidEntryPoint(path) && (
            <div style={{ flexShrink: 0 }}>
              {entryPointPath == path ? (
                <IconMapPinFilled size="0.9rem" />
              ) : (
                <IconMapPin size="0.9rem" />
              )}
            </div>
          )}
        </div>
      }
      icon={<IconComponent />}
      onClick={() => {
        onFileClick(name, path, node);
      }}
    />
  ) : (
    <TreeItem nodeId={`${path}/${name}`} label={name}>
      {Object.entries(node).map(([newName, node]) => {
        const newPath = `${path}/${name}/${newName}`;
        return (
          <FileNode
            key={newPath}
            name={newName}
            node={node}
            path={newPath}
            entryPointPath={entryPointPath}
            onFileClick={onFileClick}
          />
        );
      })}
    </TreeItem>
  );
}
