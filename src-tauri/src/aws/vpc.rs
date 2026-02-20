use aws_config::SdkConfig;
use aws_sdk_ec2::Client;

use crate::error::AppError;

pub struct VpcResources {
    pub vpc_id: String,
    pub igw_id: String,
    pub subnet_id: String,
    pub route_table_id: String,
}

pub async fn create_vpc(ec2: &Client) -> Result<String, AppError> {
    let resp = ec2
        .create_vpc()
        .cidr_block("10.0.0.0/16")
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create VPC: {}", e)))?;

    let vpc_id = resp
        .vpc()
        .and_then(|v| v.vpc_id())
        .ok_or_else(|| AppError::Aws("VPC created but no ID returned".into()))?
        .to_string();

    // Enable DNS support and hostnames
    ec2.modify_vpc_attribute()
        .vpc_id(&vpc_id)
        .enable_dns_support(aws_sdk_ec2::types::AttributeBooleanValue::builder().value(true).build())
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to enable DNS support: {}", e)))?;

    ec2.modify_vpc_attribute()
        .vpc_id(&vpc_id)
        .enable_dns_hostnames(aws_sdk_ec2::types::AttributeBooleanValue::builder().value(true).build())
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to enable DNS hostnames: {}", e)))?;

    // Tag the VPC
    tag_resource(ec2, &vpc_id, "createmyvpn-vpc").await?;

    tracing::info!("Created VPC: {}", vpc_id);
    Ok(vpc_id)
}

pub async fn create_internet_gateway(ec2: &Client, vpc_id: &str) -> Result<String, AppError> {
    let resp = ec2
        .create_internet_gateway()
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create IGW: {}", e)))?;

    let igw_id = resp
        .internet_gateway()
        .and_then(|g| g.internet_gateway_id())
        .ok_or_else(|| AppError::Aws("IGW created but no ID returned".into()))?
        .to_string();

    // Attach to VPC
    ec2.attach_internet_gateway()
        .internet_gateway_id(&igw_id)
        .vpc_id(vpc_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to attach IGW to VPC: {}", e)))?;

    tag_resource(ec2, &igw_id, "createmyvpn-igw").await?;

    tracing::info!("Created and attached IGW: {}", igw_id);
    Ok(igw_id)
}

pub async fn create_subnet(ec2: &Client, vpc_id: &str, region: &str) -> Result<String, AppError> {
    let az = format!("{}a", region);

    let resp = ec2
        .create_subnet()
        .vpc_id(vpc_id)
        .cidr_block("10.0.1.0/24")
        .availability_zone(&az)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create subnet: {}", e)))?;

    let subnet_id = resp
        .subnet()
        .and_then(|s| s.subnet_id())
        .ok_or_else(|| AppError::Aws("Subnet created but no ID returned".into()))?
        .to_string();

    // Enable auto-assign public IP
    ec2.modify_subnet_attribute()
        .subnet_id(&subnet_id)
        .map_public_ip_on_launch(aws_sdk_ec2::types::AttributeBooleanValue::builder().value(true).build())
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to enable public IP on subnet: {}", e)))?;

    tag_resource(ec2, &subnet_id, "createmyvpn-subnet").await?;

    tracing::info!("Created subnet: {}", subnet_id);
    Ok(subnet_id)
}

pub async fn create_route_table(
    ec2: &Client,
    vpc_id: &str,
    igw_id: &str,
    subnet_id: &str,
) -> Result<String, AppError> {
    let resp = ec2
        .create_route_table()
        .vpc_id(vpc_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create route table: {}", e)))?;

    let rt_id = resp
        .route_table()
        .and_then(|rt| rt.route_table_id())
        .ok_or_else(|| AppError::Aws("Route table created but no ID returned".into()))?
        .to_string();

    // Add default route to IGW
    ec2.create_route()
        .route_table_id(&rt_id)
        .destination_cidr_block("0.0.0.0/0")
        .gateway_id(igw_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create route: {}", e)))?;

    // Associate with subnet
    ec2.associate_route_table()
        .route_table_id(&rt_id)
        .subnet_id(subnet_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to associate route table: {}", e)))?;

    tag_resource(ec2, &rt_id, "createmyvpn-rt").await?;

    tracing::info!("Created route table: {}", rt_id);
    Ok(rt_id)
}

pub async fn create_all(ec2: &Client, config: &SdkConfig) -> Result<VpcResources, AppError> {
    let region = config
        .region()
        .map(|r| r.to_string())
        .unwrap_or_else(|| "us-east-1".to_string());

    let vpc_id = create_vpc(ec2).await?;
    let igw_id = create_internet_gateway(ec2, &vpc_id).await?;
    let subnet_id = create_subnet(ec2, &vpc_id, &region).await?;
    let route_table_id = create_route_table(ec2, &vpc_id, &igw_id, &subnet_id).await?;

    Ok(VpcResources {
        vpc_id,
        igw_id,
        subnet_id,
        route_table_id,
    })
}

async fn tag_resource(ec2: &Client, resource_id: &str, name: &str) -> Result<(), AppError> {
    ec2.create_tags()
        .resources(resource_id)
        .tags(
            aws_sdk_ec2::types::Tag::builder()
                .key("Name")
                .value(name)
                .build(),
        )
        .tags(
            aws_sdk_ec2::types::Tag::builder()
                .key("ManagedBy")
                .value("createmyvpn")
                .build(),
        )
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to tag {}: {}", resource_id, e)))?;

    Ok(())
}
