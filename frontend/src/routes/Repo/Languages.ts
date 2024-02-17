import {
  IconAlignLeft,
  IconBraces,
  IconBrandRust,
  IconFile,
  IconSettings,
} from '@tabler/icons-react';

export function getLanguageInfo(extension: string) {
  switch (extension) {
    case 'rs':
      return {
        monacoLanguage: 'rust',
        iconComponent: IconBrandRust,
        iconColor: '#6d8086',
      };
    case 'json':
      return {
        monacoLanguage: 'json',
        iconComponent: IconBraces,
        iconColor: '#cbcb41',
      };
    case 'toml':
      return {
        monacoLanguage: 'toml',
        iconComponent: IconSettings,
        iconColor: '#6d8086',
      };
    case 'lock':
      return {
        monacoLanguage: 'lock',
        iconComponent: IconAlignLeft,
        iconColor: '#6d8086',
      };
    default:
      return {
        monacoLanguage: 'plaintext',
        iconComponent: IconFile,
        iconColor: '#6d8086',
      };
  }
}
