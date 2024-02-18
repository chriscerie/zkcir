use std::sync::Arc;

use aws_config::SdkConfig;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    aws_config: SdkConfig,
    dynamodb_client: Arc<aws_sdk_dynamodb::Client>,
    s3_client: Arc<aws_sdk_s3::Client>,
    lambda_client: Arc<aws_sdk_lambda::Client>,
    iam_client: Arc<aws_sdk_iam::Client>,
    codecommit_client: Arc<aws_sdk_codecommit::Client>,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(aws_config: SdkConfig) -> Self {
        let dynamodb_client = aws_sdk_dynamodb::Client::new(&aws_config);
        let s3_client = aws_sdk_s3::Client::new(&aws_config);
        let lambda_client = aws_sdk_lambda::Client::new(&aws_config);
        let iam_client = aws_sdk_iam::Client::new(&aws_config);
        let codecommit_client = aws_sdk_codecommit::Client::new(&aws_config);

        Self {
            aws_config,
            dynamodb_client: Arc::new(dynamodb_client),
            s3_client: Arc::new(s3_client),
            lambda_client: Arc::new(lambda_client),
            iam_client: Arc::new(iam_client),
            codecommit_client: Arc::new(codecommit_client),
        }
    }

    pub fn get_aws_config(&self) -> &SdkConfig {
        &self.aws_config
    }

    pub fn get_dynamodb_client(&self) -> &aws_sdk_dynamodb::Client {
        &self.dynamodb_client
    }

    pub fn get_s3_client(&self) -> &aws_sdk_s3::Client {
        &self.s3_client
    }

    pub fn get_lambda_client(&self) -> &aws_sdk_lambda::Client {
        &self.lambda_client
    }

    pub fn get_iam_client(&self) -> &aws_sdk_iam::Client {
        &self.iam_client
    }

    pub fn get_codecommit_client(&self) -> &aws_sdk_codecommit::Client {
        &self.codecommit_client
    }
}
