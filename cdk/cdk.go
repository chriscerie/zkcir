package main

import (
	"github.com/aws/aws-cdk-go/awscdk/v2"
	"github.com/aws/aws-cdk-go/awscdk/v2/awscertificatemanager"
	"github.com/aws/aws-cdk-go/awscdk/v2/awscognito"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsdynamodb"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsec2"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsecrassets"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsecs"
	"github.com/aws/aws-cdk-go/awscdk/v2/awselasticloadbalancingv2"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsiam"
	"github.com/aws/aws-cdk-go/awscdk/v2/awslambda"
	"github.com/aws/aws-cdk-go/awscdk/v2/awslogs"
	"github.com/aws/aws-cdk-go/awscdk/v2/awss3"
	"github.com/aws/aws-cdk-go/awscdk/v2/awssecretsmanager"
	"github.com/aws/constructs-go/constructs/v10"
	"github.com/aws/jsii-runtime-go"
)

const (
	ACCOUNT_ID       = "388179654007"
	REGION           = "us-east-1"
	GOOGLE_CLIENT_ID = "585833256001-k7otn8at0hvvttmug2le7nm0mm4ehfvb.apps.googleusercontent.com"
	USER_POOL_DOMAIN = "zkcir"
	DOMAIN_NAME      = "zkcir.chrisc.dev"
)

type ZkcirCdkStackProps struct {
	awscdk.StackProps
}

func NewZkcirCdkStack(scope constructs.Construct, id string, props *ZkcirCdkStackProps) awscdk.Stack {
	var sprops awscdk.StackProps
	if props != nil {
		sprops = props.StackProps
	}
	stack := awscdk.NewStack(scope, &id, &sprops)

	// Ideally validation uses DNS, but deploying times out too quickly for unknown reason.
	certificate := awscertificatemanager.NewCertificate(stack, jsii.String("Certificate"), &awscertificatemanager.CertificateProps{
		DomainName: jsii.String(DOMAIN_NAME),
	})

	userPool := awscognito.NewUserPool(stack, jsii.String("ZkcirUserPool"), &awscognito.UserPoolProps{
		SelfSignUpEnabled: jsii.Bool(true),
		AutoVerify: &awscognito.AutoVerifiedAttrs{
			Email: jsii.Bool(true),
			Phone: jsii.Bool(false),
		},
		RemovalPolicy: awscdk.RemovalPolicy_DESTROY,
	})

	googleProvider := awscognito.NewUserPoolIdentityProviderGoogle(stack, jsii.String("ZkcirGoogleProvider"), &awscognito.UserPoolIdentityProviderGoogleProps{
		ClientId:          jsii.String(GOOGLE_CLIENT_ID),
		ClientSecretValue: awssecretsmanager.Secret_FromSecretNameV2(stack, jsii.String("GoogleClientSecret"), jsii.String("GoogleClientSecret")).SecretValue(),
		UserPool:          userPool,
		AttributeMapping: &awscognito.AttributeMapping{
			Email:          awscognito.ProviderAttribute_GOOGLE_EMAIL(),
			Fullname:       awscognito.ProviderAttribute_GOOGLE_NAME(),
			GivenName:      awscognito.ProviderAttribute_GOOGLE_GIVEN_NAME(),
			ProfilePicture: awscognito.ProviderAttribute_GOOGLE_PICTURE(),
		},
	})

	awscognito.NewUserPoolDomain(stack, jsii.String("ZkcirUserPoolDomain"), &awscognito.UserPoolDomainProps{
		UserPool: userPool,
		CognitoDomain: &awscognito.CognitoDomainOptions{
			DomainPrefix: jsii.String(USER_POOL_DOMAIN),
		},
	})

	vpc := awsec2.NewVpc(stack, jsii.String("Vpc"), &awsec2.VpcProps{
		MaxAzs:      jsii.Number(2),
		NatGateways: jsii.Number(1),
		SubnetConfiguration: &[]*awsec2.SubnetConfiguration{
			{
				Name:       jsii.String("public"),
				SubnetType: awsec2.SubnetType_PUBLIC,
			},
			{
				Name:       jsii.String("private"),
				SubnetType: awsec2.SubnetType_PRIVATE_WITH_NAT,
			},
		},
		GatewayEndpoints: &map[string]*awsec2.GatewayVpcEndpointOptions{
			"S3Endpoint": {
				Service: awsec2.GatewayVpcEndpointAwsService_S3(),
			},
			"DynamoDBEndpoint": {
				Service: awsec2.GatewayVpcEndpointAwsService_DYNAMODB(),
			},
		},
	})

	axum_image := awsecrassets.NewDockerImageAsset(stack, jsii.String("AxumImage"), &awsecrassets.DockerImageAssetProps{
		Directory: jsii.String(".."),
		Exclude:   jsii.Strings("cdk"),
		Target:    jsii.String("core-ecs"),
	})

	cluster := awsecs.NewCluster(stack, jsii.String("Cluster"), &awsecs.ClusterProps{
		Vpc: vpc,
	})

	// Permission for ECS to pull docker image from ECR
	executionRole := awsiam.NewRole(stack, jsii.String("ExecutionRole"), &awsiam.RoleProps{
		AssumedBy: awsiam.NewServicePrincipal(jsii.String("ecs-tasks.amazonaws.com"), nil),
	})
	executionRole.AddManagedPolicy(awsiam.ManagedPolicy_FromAwsManagedPolicyName(jsii.String("service-role/AmazonECSTaskExecutionRolePolicy")))
	axum_image.Repository().GrantPull(executionRole)

	taskDefinition := awsecs.NewFargateTaskDefinition(stack, jsii.String("TaskDef"), &awsecs.FargateTaskDefinitionProps{
		MemoryLimitMiB: jsii.Number(2048),
		Cpu:            jsii.Number(512),
		ExecutionRole:  executionRole,
	})

	container := taskDefinition.AddContainer(jsii.String("AxumContainer"), &awsecs.ContainerDefinitionOptions{
		Image: awsecs.ContainerImage_FromDockerImageAsset(axum_image),
		Logging: awsecs.LogDriver_AwsLogs(&awsecs.AwsLogDriverProps{
			StreamPrefix: jsii.String("Service"),
			LogRetention: awslogs.RetentionDays_ONE_WEEK,
		}),
	})

	compileLambda := awslambda.NewFunction(stack, jsii.String("CompileLambda"), &awslambda.FunctionProps{
		Code: awslambda.Code_FromAssetImage(jsii.String(".."), &awslambda.AssetImageCodeProps{
			Exclude: jsii.Strings("cdk"),
			Target:  jsii.String("compile-lambda"),
		}),
		Handler:              awslambda.Handler_FROM_IMAGE(),
		Runtime:              awslambda.Runtime_FROM_IMAGE(),
		MemorySize:           jsii.Number(1024),
		EphemeralStorageSize: awscdk.Size_Gibibytes(jsii.Number(2)),
		LogRetention:         awslogs.RetentionDays_ONE_WEEK,
		Timeout:              awscdk.Duration_Seconds(jsii.Number(30)),
	})

	compileLambda.GrantInvoke(taskDefinition.TaskRole())

	container.AddPortMappings(&awsecs.PortMapping{
		ContainerPort: jsii.Number(3000),
	})

	fargateService := awsecs.NewFargateService(stack, jsii.String("Service"), &awsecs.FargateServiceProps{
		Cluster:        cluster,
		TaskDefinition: taskDefinition,
	})

	lb := awselasticloadbalancingv2.NewApplicationLoadBalancer(stack, jsii.String("ALB"), &awselasticloadbalancingv2.ApplicationLoadBalancerProps{
		Vpc:            vpc,
		InternetFacing: jsii.Bool(true),
	})

	httpsListener := lb.AddListener(jsii.String("HttpsListener"), &awselasticloadbalancingv2.BaseApplicationListenerProps{
		Port: jsii.Number(443),
		Certificates: &[]awselasticloadbalancingv2.IListenerCertificate{
			awselasticloadbalancingv2.ListenerCertificate_FromCertificateManager(certificate),
		},
	})

	httpsListener.AddTargets(jsii.String("ECSHttps"), &awselasticloadbalancingv2.AddApplicationTargetsProps{
		Port:     jsii.Number(3000),
		Targets:  &[]awselasticloadbalancingv2.IApplicationLoadBalancerTarget{fargateService},
		Protocol: awselasticloadbalancingv2.ApplicationProtocol_HTTP,
	})

	userPoolClient := awscognito.NewUserPoolClient(stack, jsii.String("ZkcirUserPoolClient"), &awscognito.UserPoolClientProps{
		UserPool: userPool,
		SupportedIdentityProviders: &[]awscognito.UserPoolClientIdentityProvider{
			awscognito.UserPoolClientIdentityProvider_GOOGLE(),
		},
		OAuth: &awscognito.OAuthSettings{
			CallbackUrls: jsii.Strings("https://"+DOMAIN_NAME+"/auth/callback", "http://localhost:3000/auth/callback", "http://localhost:8000/auth/callback"),
			Scopes: &[]awscognito.OAuthScope{
				awscognito.OAuthScope_EMAIL(),
				awscognito.OAuthScope_OPENID(),
				awscognito.OAuthScope_PROFILE(),
				awscognito.OAuthScope_COGNITO_ADMIN(),
			},
		},
		AccessTokenValidity: awscdk.Duration_Days(jsii.Number(1)),
	})

	// Otherwise errors `The provider Google does not exist for User Pool`
	userPoolClient.Node().AddDependency(googleProvider)

	usersTable := awsdynamodb.NewTable(stack, jsii.String("Users"), &awsdynamodb.TableProps{
		BillingMode: awsdynamodb.BillingMode_PROVISIONED,
		PartitionKey: &awsdynamodb.Attribute{
			Name: jsii.String("user_id"),
			Type: awsdynamodb.AttributeType_STRING,
		},
		RemovalPolicy: awscdk.RemovalPolicy_DESTROY,
		WriteCapacity: jsii.Number(25),
		ReadCapacity:  jsii.Number(25),
	})
	usersTable.GrantReadWriteData(taskDefinition.TaskRole())

	circuitsBucket := awss3.NewBucket(stack, jsii.String("Circuits"), &awss3.BucketProps{
		AutoDeleteObjects: jsii.Bool(true),
		RemovalPolicy:     awscdk.RemovalPolicy_DESTROY,
	})
	circuitsBucket.GrantReadWrite(taskDefinition.TaskRole(), nil)

	// ECS needs these, but they're generated by cdk
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("ddb_user_table"), usersTable.TableName())
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("circuits_bucket"), circuitsBucket.BucketName())
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("user_pool_domain"), jsii.String(USER_POOL_DOMAIN))
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("user_pool_client_id"), userPoolClient.UserPoolClientId())
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("aws_region"), jsii.String(REGION))
	taskDefinition.DefaultContainer().AddEnvironment(jsii.String("compile_lambda_arn"), compileLambda.FunctionArn())

	return stack
}

func main() {
	defer jsii.Close()

	app := awscdk.NewApp(nil)

	NewZkcirCdkStack(app, "ZkcirCdkStack", &ZkcirCdkStackProps{
		awscdk.StackProps{
			Env: env(),
		},
	})

	app.Synth(nil)
}

// https://docs.aws.amazon.com/cdk/latest/guide/environments.html
func env() *awscdk.Environment {
	return &awscdk.Environment{
		Account: jsii.String(ACCOUNT_ID),
		Region:  jsii.String(REGION),
	}
}
