export type ListReposResponse = {
  repos: Array<{
    name: string;
  }>;
};

export type GetRepoMetadataResponse = {
  description: string;
  clone_url_ssh: string;
  latest_commit_id?: string;
};

export type GetIrResponse = {
  json?: string;
  cir?: string;
  status: GetIrStatusResponse;
};

export type GetIrVersionsResponse = {
  // Array of uuid v7 strings
  versions: Array<string>;
};

export type GetIrSourceResponse = Blob;

export enum GetIrStatusResponse {
  NotStarted = 'NotStarted', // Not actually returned
  CloningRepository = 'CloningRepository',
  Compiling = 'Compiling',
  Completed = 'Completed',
}

export type ListKeysResponse = {
  keys: Array<{
    id: string;
    is_active: boolean;

    // ISO 8601 date-time format
    upload_time: string;
  }>;
};
