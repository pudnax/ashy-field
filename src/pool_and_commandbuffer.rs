use crate::instance_device_queues::QueueFamilies;
use ash::{version::DeviceV1_0, vk};
use eyre::*;

pub struct Pools {
    pub commandpool_graphics: vk::CommandPool,
    commandpool_transfer: vk::CommandPool,
}

impl Pools {
    pub fn init(logical_device: &ash::Device, queue_families: &QueueFamilies) -> Result<Self> {
        let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_graphics =
            unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
        let transfer_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.transfer_q_index.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_transfer =
            unsafe { logical_device.create_command_pool(&transfer_commandpool_info, None) }?;

        Ok(Pools {
            commandpool_graphics,
            commandpool_transfer,
        })
    }
    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.commandpool_graphics, None);
            logical_device.destroy_command_pool(self.commandpool_transfer, None);
        }
    }
}

pub fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: u32,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}
