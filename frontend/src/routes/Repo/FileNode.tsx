import 'allotment/dist/style.css';
import { TreeItem } from '@mui/x-tree-view';
import { getLanguageInfo } from './Languages';

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
  onFileClick: (fileName: string, path: string, contents: string) => void;
}) {
  const IconComponent = getLanguageInfo(
    name.split('.').pop() || 'txt',
  ).iconComponent;

  return typeof node == 'string' ? (
    <TreeItem
      nodeId={`${path}/${name}`}
      label={name}
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
            onFileClick={onFileClick}
          />
        );
      })}
    </TreeItem>
  );
}
