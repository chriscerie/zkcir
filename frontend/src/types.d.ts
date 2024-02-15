export type ListIrsMetadataResponse = {
  irs: [
    {
      repo_name: string;
      circuit_version: string;
      name: string;
      description: string;
    },
  ];
};
