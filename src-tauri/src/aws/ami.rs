use aws_config::SdkConfig;
use aws_sdk_ec2::types::Filter;

use crate::error::AppError;

/// Look up the latest Ubuntu 22.04 LTS AMI.
///
/// Strategy:
///   1. Try SSM Parameter Store (fast, no extra permissions needed on most setups).
///   2. Fall back to EC2 DescribeImages with Canonical's official owner + name filter.
pub async fn lookup_ubuntu_ami(config: &SdkConfig) -> Result<String, AppError> {
    match lookup_via_ssm(config).await {
        Ok(ami_id) => Ok(ami_id),
        Err(ssm_err) => {
            tracing::warn!(
                "SSM AMI lookup failed ({}), falling back to EC2 DescribeImages",
                ssm_err
            );
            lookup_via_describe_images(config).await
        }
    }
}

async fn lookup_via_ssm(config: &SdkConfig) -> Result<String, AppError> {
    let ssm_client = aws_sdk_ssm::Client::new(config);
    let param_name =
        "/aws/service/canonical/ubuntu/server/22.04/stable/current/amd64/hvm/ebs-gp2/ami-id";

    tracing::info!("Looking up Ubuntu 22.04 AMI via SSM: {}", param_name);

    let resp = ssm_client
        .get_parameter()
        .name(param_name)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("SSM GetParameter failed: {}", e);
            tracing::debug!("{}", msg);
            AppError::Aws(msg)
        })?;

    let ami_id = resp
        .parameter()
        .and_then(|p| p.value())
        .ok_or_else(|| AppError::Aws("SSM parameter returned no value".into()))?
        .to_string();

    tracing::info!("Resolved Ubuntu 22.04 AMI via SSM: {}", ami_id);
    Ok(ami_id)
}

async fn lookup_via_describe_images(config: &SdkConfig) -> Result<String, AppError> {
    // Canonical's official AWS account ID — same in all regions.
    let canonical_owner = "099720109477";
    let name_glob = "ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*";

    tracing::info!(
        "Searching for Ubuntu 22.04 AMI via EC2 DescribeImages (owner: {}, name: {})",
        canonical_owner,
        name_glob
    );

    let ec2_client = aws_sdk_ec2::Client::new(config);

    let resp = ec2_client
        .describe_images()
        .owners(canonical_owner)
        .filters(
            Filter::builder()
                .name("name")
                .values(name_glob)
                .build(),
        )
        .filters(
            Filter::builder()
                .name("state")
                .values("available")
                .build(),
        )
        .filters(
            Filter::builder()
                .name("architecture")
                .values("x86_64")
                .build(),
        )
        .filters(
            Filter::builder()
                .name("root-device-type")
                .values("ebs")
                .build(),
        )
        .filters(
            Filter::builder()
                .name("virtualization-type")
                .values("hvm")
                .build(),
        )
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("EC2 DescribeImages failed: {}", e)))?;

    let mut images = resp.images().to_vec();
    tracing::info!("DescribeImages returned {} candidates", images.len());

    if images.is_empty() {
        return Err(AppError::Aws(
            "No Ubuntu 22.04 AMIs found in this region via DescribeImages".into(),
        ));
    }

    // Sort by creation date descending, pick the newest.
    images.sort_by(|a, b| {
        b.creation_date()
            .unwrap_or("")
            .cmp(a.creation_date().unwrap_or(""))
    });

    let image = &images[0];
    let ami_id = image
        .image_id()
        .ok_or_else(|| AppError::Aws("AMI found but has no image ID".into()))?
        .to_string();

    tracing::info!(
        "Resolved Ubuntu 22.04 AMI via DescribeImages: {} (name: {}, created: {})",
        ami_id,
        image.name().unwrap_or("unknown"),
        image.creation_date().unwrap_or("unknown")
    );

    Ok(ami_id)
}
