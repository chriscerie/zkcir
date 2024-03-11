use std::sync::Arc;

use aws_config::SdkConfig;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    aws_config: SdkConfig,
    pub dynamodb_client: Arc<aws_sdk_dynamodb::Client>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub lambda_client: Arc<aws_sdk_lambda::Client>,
    pub iam_client: Arc<aws_sdk_iam::Client>,
    pub codecommit_client: Arc<aws_sdk_codecommit::Client>,
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
}
