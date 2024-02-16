export type ListIrsMetadataResponse = {
  irs: Array<{
    repo_name: string;
    description: string;
  }>;
};

export type GetIrJsonResponse = {
  ir: string;
};

export type GetIrVersionsResponse = {
  // Array of uuid v7 strings
  versions: Array<string>;
};
