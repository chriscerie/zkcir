import { minimatch } from 'minimatch';

export function isValidEntryPoint(path: string) {
  return minimatch(path, 'examples/**');
}
