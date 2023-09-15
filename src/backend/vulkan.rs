use anyhow::{Context, Result};
use drm_fourcc::{DrmFourcc, DrmModifier};
use smithay::{
    backend::{
        allocator::{
            dmabuf::AsDmabuf,
            vulkan::{ImageUsageFlags, VulkanAllocator},
            Allocator, Buffer,
        },
        vulkan::{version::Version, Instance, PhysicalDevice},
    },
    reexports::calloop::EventLoop,
};

pub fn run(
    _event_loop: &mut EventLoop<crate::LoopData>,
    _data: &mut crate::LoopData,
) -> Result<()> {
    log::info!(
        "Available instance extensions: {:?}",
        Instance::enumerate_extensions()?.collect::<Vec<_>>()
    );

    let instance = Instance::new(Version::VERSION_1_3, None)?;

    for (idx, phy) in PhysicalDevice::enumerate(&instance)?.enumerate() {
        log::info!(
            "Device #{}: {} v{}, {:?}",
            idx,
            phy.name(),
            phy.api_version(),
            phy.driver()
        );
    }

    let physical_device = PhysicalDevice::enumerate(&instance)?
        .next()
        .with_context(|| anyhow::anyhow!("No physical devices"))?;

    log::debug!(
        "Required extensions by VulkanAllocator: {:?}",
        VulkanAllocator::required_extensions(&physical_device)
    );

    // The allocator should create buffers that are suitable as render targets.
    let mut allocator = VulkanAllocator::new(&physical_device, ImageUsageFlags::COLOR_ATTACHMENT)?;

    let image = allocator.create_buffer(100, 200, DrmFourcc::Argb8888, &[DrmModifier::Linear])?;

    assert_eq!(image.width(), 100);
    assert_eq!(image.height(), 200);

    let image_dmabuf = image.export()?;

    drop(image);

    let _image2 = allocator.create_buffer(200, 200, DrmFourcc::Argb8888, &[DrmModifier::Linear])?;

    drop(allocator);
    drop(image_dmabuf);

    Ok(())
}
