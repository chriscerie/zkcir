import {
  IconAlignLeft,
  IconBrandRust,
  IconFile,
  IconJson,
  IconSettings,
} from '@tabler/icons-react';
import 'allotment/dist/style.css';
import { TreeItem } from '@mui/x-tree-view';

export interface IFileNode {
  // [file name]: source | FileNode
  [key: string]: string | IFileNode;
}

export default function FileNode({
  name,
  node,
  path,
  onFileClick,
}: {
  name: string;
  node: string | IFileNode;
  // Not including leading slash
  path: string;
  onFileClick: (path: string, contents: string) => void;
}) {
  const icon = (() => {
    switch (true) {
      case name.endsWith('.rs'):
        return <IconBrandRust color="#6d8086" stroke={2} />;
      case name.endsWith('.json'):
        return <IconJson color="#6d8086" />;
      case name.endsWith('.toml'):
        return <IconSettings color="#6d8086" />;
      case name.endsWith('.lock'):
        return <IconAlignLeft />;
      default:
        return <IconFile />;
    }
  })();

  return typeof node == 'string' ? (
    <TreeItem
      nodeId={`${path}/${name}`}
      label={name}
      icon={icon}
      onClick={() => {
        onFileClick(path, node);
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
            onFileClick={onFileClick}
          />
        );
      })}
    </TreeItem>
  );
}
