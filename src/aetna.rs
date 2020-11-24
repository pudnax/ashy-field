use crate::{
    buffers::Buffer,
    debug::DebugDongXi,
    instance_device_queues::{
        init_device_and_queues, init_instance, init_physical_device_and_properties, QueueFamilies,
        Queues,
    },
    model::Model,
    pool_and_commandbuffer::{create_commandbuffers, Pools},
    renderpass_and_pipeline::{init_renderpass, Pipeline},
    surface::SurfaceDongXi,
    swapchain::SwapchainDongXi,
};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use eyre::*;
use nalgebra as na;

// TODO(#3): Rethink about the order of poles in the struct for 'right' drop order
// to remove ManualDrop
pub struct Aetna<V, I> {
    pub window: winit::window::Window,
    _entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugDongXi>,
    surfaces: std::mem::ManuallyDrop<SurfaceDongXi>,
    physical_device: vk::PhysicalDevice,
    _physical_device_properties: vk::PhysicalDeviceProperties,
    _physical_device_features: vk::PhysicalDeviceFeatures,
    pub queue_families: QueueFamilies,
    pub queues: Queues,
    pub device: ash::Device,
    pub swapchain: SwapchainDongXi,
    renderpass: vk::RenderPass,
    pipeline: Pipeline,
    pub pools: Pools,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub allocator: vk_mem::Allocator,
    pub models: Vec<Model<V, I>>,
    pub uniformbuffer: Buffer,
    pub lightbuffer: Buffer,
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets_camera: Vec<vk::DescriptorSet>,
    pub descriptor_sets_light: Vec<vk::DescriptorSet>,
}

impl<V, I> Aetna<V, I> {
    pub fn init(window: winit::window::Window) -> Result<Self> {
        let entry = ash::Entry::new()?;
        let extension_names = ash_window::enumerate_required_extensions(&window)?;

        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = init_instance(&entry, &layer_names, &extension_names)?;
        let debug = DebugDongXi::init(&entry, &instance)?;
        let surfaces = SurfaceDongXi::init(&window, &entry, &instance)?;

        let (physical_device, physical_device_properties, physical_device_features) =
            init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families, &layer_names)?;

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.clone(),
            heap_size_limits: None,
            frame_in_use_count: 0,
            preferred_large_heap_block_size: 0,
            flags: vk_mem::AllocatorCreateFlags::NONE,
        };
        let allocator = vk_mem::Allocator::new(&allocator_create_info)?;

        let mut swapchain = SwapchainDongXi::init(
            &instance,
            physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &allocator,
        )?;
        let renderpass = init_renderpass(&logical_device, swapchain.surface_format.format)?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;
        let pipeline = Pipeline::init(&logical_device, &swapchain, &renderpass)?;
        let pools = Pools::init(&logical_device, &queue_families)?;

        let commandbuffers =
            create_commandbuffers(&logical_device, &pools, swapchain.amount_of_images)?;

        let mut uniformbuffer = Buffer::new(
            &allocator,
            64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        let cameratransforms: [[[f32; 4]; 4]; 2] = [
            na::Matrix4::identity().into(),
            na::Matrix4::identity().into(),
        ];
        uniformbuffer.fill(&allocator, &cameratransforms)?;

        let mut lightbuffer = Buffer::new(
            &allocator,
            8,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        lightbuffer.fill(&allocator, &[0., 0.])?;

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(2 * swapchain.amount_of_images)
            .pool_sizes(&pool_sizes);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;

        let desc_layouts_camera =
            vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_camera = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_camera);
        let descriptor_sets_camera = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_camera)
        }?;

        for descset in &descriptor_sets_camera {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniformbuffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }
        let desc_layouts_light =
            vec![pipeline.descriptor_set_layouts[1]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_light = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_light);
        let descriptor_sets_light = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_light)
        }?;

        for descset in &descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: lightbuffer.buffer,
                offset: 0,
                range: 8,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }

        Ok(Aetna {
            window,
            _entry: entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surfaces: std::mem::ManuallyDrop::new(surfaces),
            physical_device,
            _physical_device_properties: physical_device_properties,
            _physical_device_features: physical_device_features,
            queue_families,
            queues,
            device: logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            commandbuffers,
            allocator,
            models: vec![],
            uniformbuffer,
            lightbuffer,
            descriptor_pool,
            descriptor_sets_camera,
            descriptor_sets_light,
        })
    }
    // TODO(#4): Still have validation errors on validation.
    pub fn recreate_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        let _extent2d = vk::Extent2D { width, height };
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");
        }
        unsafe {
            self.swapchain.cleanup(&self.device, &self.allocator);
        }
        self.swapchain = SwapchainDongXi::init(
            &self.instance,
            self.physical_device,
            &self.device,
            &self.surfaces,
            &self.queue_families,
            &self.allocator,
        )?;
        self.swapchain
            .create_framebuffers(&self.device, self.renderpass)?;
        self.pipeline.cleanup(&self.device);
        self.pipeline = Pipeline::init(&self.device, &self.swapchain, &self.renderpass)?;
        Ok(())
    }
    pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
        let commandbuffer = self.commandbuffers[index];
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }
        let clearvalues = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.08, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            self.device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[
                    self.descriptor_sets_camera[index],
                    self.descriptor_sets_light[index],
                ],
                &[],
            );
            for m in &self.models {
                m.draw(&self.device, commandbuffer);
            }
            self.device.cmd_end_render_pass(commandbuffer);
            self.device.end_command_buffer(commandbuffer)?;
        }
        Ok(())
    }
}

impl<V, I> Drop for Aetna<V, I> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Something went wrong while waiting.");
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.allocator
                .destroy_buffer(self.lightbuffer.buffer, &self.lightbuffer.allocation)
                .expect("buffer destruction");
            self.allocator
                .destroy_buffer(self.uniformbuffer.buffer, &self.uniformbuffer.allocation)
                .expect("Failed destroy uniform buffer");
            for m in &self.models {
                if let Some(vb) = &m.vertexbuffer {
                    self.allocator
                        .destroy_buffer(vb.buffer, &vb.allocation)
                        .expect("problem with buffer destruction");
                }
                if let Some(ib) = &m.instancebuffer {
                    self.allocator
                        .destroy_buffer(ib.buffer, &ib.allocation)
                        .expect("problem with buffer destruction");
                }
                if let Some(ib) = &m.indexbuffer {
                    self.allocator
                        .destroy_buffer(ib.buffer, &ib.allocation)
                        .expect("Failed destroy index buffer.")
                }
            }
            self.pools.cleanup(&self.device);
            self.pipeline.cleanup(&self.device);
            self.device.destroy_render_pass(self.renderpass, None);
            self.swapchain.cleanup(&self.device, &self.allocator);
            self.allocator.destroy();
            self.device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
