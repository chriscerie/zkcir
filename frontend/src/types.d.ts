export type ListIrsMetadataResponse = {
  irs: Array<{
    repo_name: string;
    description: string;
  }>;
};

export type GetIrResponse = {
  json: string;
  cir: string;
};

export type GetIrVersionsResponse = {
  // Array of uuid v7 strings
  versions: Array<string>;
};

export type GetIrSourceResponse = Blob;
