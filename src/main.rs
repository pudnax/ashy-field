use ash::{version::DeviceV1_0, vk};
use eyre::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use nalgebra as na;

mod aetna;
mod angle;
mod buffers;
mod camera;
mod debug;
mod instance_device_queues;
mod math;
mod model;
mod pool_and_commandbuffer;
mod renderpass_and_pipeline;
mod surface;
mod swapchain;
mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;
    let eventloop = EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
    let mut aetna = aetna::Aetna::init(window)?;
    let mut sphere = model::Model::<model::VertexData, model::InstanceData>::sphere(3);
    sphere.insert_visibly(model::InstanceData::from_matrix_and_colour(
        na::Matrix4::new_scaling(0.5),
        [0.955, 0.638, 0.538],
    ));
    sphere.update_vertexbuffer(&aetna.allocator)?;
    sphere.update_indexbuffer(&aetna.allocator)?;
    sphere.update_instancebuffer(&aetna.allocator)?;
    aetna.models = vec![sphere];
    let mut camera = camera::Camera::builder().build();

    let mut shift_acceleration = 0.;
    eventloop.run(move |event, _, controlflow| {
        *controlflow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *controlflow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                let shift_pressed = state.shift() as u32;
                shift_acceleration = 0.2 * shift_pressed as f32;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                } = input
                {
                    match keycode {
                        VirtualKeyCode::Right | VirtualKeyCode::D => {
                            camera.turn_right(0.1 + shift_acceleration);
                        }
                        VirtualKeyCode::Left | VirtualKeyCode::A => {
                            camera.turn_left(0.1 + shift_acceleration);
                        }
                        VirtualKeyCode::Up | VirtualKeyCode::W => {
                            camera.move_forward(0.05 + shift_acceleration);
                        }
                        VirtualKeyCode::Down | VirtualKeyCode::S => {
                            camera.move_backward(0.05 + shift_acceleration);
                        }
                        VirtualKeyCode::Space => {
                            camera.turn_down(0.02);
                        }
                        VirtualKeyCode::Z => {
                            camera.turn_up(0.02);
                        }
                        VirtualKeyCode::Escape => {
                            *controlflow = ControlFlow::Exit;
                        }
                        VirtualKeyCode::F12 => {
                            screenshot(&aetna).expect("screenshot trouble");
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                aetna.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                aetna
                    .recreate_swapchain(new_size.width, new_size.height)
                    .expect("Failed recreate swapchain.");
                // camera.set_aspect(
                //     aetna.swapchain.extent.width as f32 / aetna.swapchain.extent.height as f32,
                // );
                // camera
                //     .update_buffer(&aetna.allocator, &mut aetna.uniformbuffer)
                //     .expect("camera buffer update");
            }

            Event::RedrawRequested(_) => {
                let (image_index, _) = unsafe {
                    aetna
                        .swapchain
                        .swapchain_loader
                        .acquire_next_image(
                            aetna.swapchain.swapchain,
                            std::u64::MAX,
                            aetna.swapchain.image_available[aetna.swapchain.current_image],
                            vk::Fence::null(),
                        )
                        .expect("image acquisition trouble")
                };
                unsafe {
                    aetna
                        .device
                        .wait_for_fences(
                            &[aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]],
                            true,
                            std::u64::MAX,
                        )
                        .expect("fence-waiting");
                    aetna
                        .device
                        .reset_fences(&[
                            aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]
                        ])
                        .expect("resetting fences");
                }
                camera
                    .update_buffer(&aetna.allocator, &mut aetna.uniformbuffer)
                    .expect("Failed update camera buffer.");
                for m in &mut aetna.models {
                    m.update_instancebuffer(&aetna.allocator)
                        .expect("Failed update instance buffer");
                }
                aetna
                    .update_commandbuffer(image_index as usize)
                    .expect("updating the command buffer");
                let semaphores_available =
                    [aetna.swapchain.image_available[aetna.swapchain.current_image]];
                let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                let semaphores_finished =
                    [aetna.swapchain.rendering_finished[aetna.swapchain.current_image]];
                let commandbuffers = [aetna.commandbuffers[image_index as usize]];
                let submit_info = [vk::SubmitInfo::builder()
                    .wait_semaphores(&semaphores_available)
                    .wait_dst_stage_mask(&waiting_stages)
                    .command_buffers(&commandbuffers)
                    .signal_semaphores(&semaphores_finished)
                    .build()];
                unsafe {
                    aetna
                        .device
                        .queue_submit(
                            aetna.queues.graphics_queue,
                            &submit_info,
                            aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image],
                        )
                        .expect("queue submission");
                };
                let swapchains = [aetna.swapchain.swapchain];
                let indices = [image_index];
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&semaphores_finished)
                    .swapchains(&swapchains)
                    .image_indices(&indices);
                unsafe {
                    match aetna
                        .swapchain
                        .swapchain_loader
                        .queue_present(aetna.queues.graphics_queue, &present_info)
                    {
                        Ok(..) => {}
                        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            aetna
                                .recreate_swapchain(0, 0)
                                .expect("Failed recreate swapchain.");
                            camera.set_aspect(
                                aetna.swapchain.extent.width as f32
                                    / aetna.swapchain.extent.height as f32,
                            );
                            camera
                                .update_buffer(&aetna.allocator, &mut aetna.uniformbuffer)
                                .expect("camera buffer update");
                        }
                        _ => panic!("Unhandled queue presentation error."),
                    }
                };
                aetna.swapchain.current_image =
                    (aetna.swapchain.current_image + 1) % aetna.swapchain.amount_of_images as usize;
            }
            _ => {}
        }
    });
}

// TODO(#6): Allocate commandbuffers beforehand.
fn screenshot<V, I>(aetna: &aetna::Aetna<V, I>) -> Result<(), Box<dyn std::error::Error>> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(aetna.pools.commandpool_graphics)
        .command_buffer_count(1);
    let copybuffer = unsafe {
        aetna
            .device
            .allocate_command_buffers(&commandbuf_allocate_info)
    }?[0];

    let cmdbegininfo =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { aetna.device.begin_command_buffer(copybuffer, &cmdbegininfo) }?;

    let ici = vk::ImageCreateInfo::builder()
        .format(vk::Format::R8G8B8A8_UNORM)
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: aetna.swapchain.extent.width,
            height: aetna.swapchain.extent.height,
            depth: 1,
        })
        .array_layers(1)
        .mip_levels(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::LINEAR)
        .usage(vk::ImageUsageFlags::TRANSFER_DST)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocinfo = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuToCpu,
        ..Default::default()
    };
    let (destination_image, dst_alloc, _allocinfo) =
        aetna.allocator.create_image(&ici, &allocinfo)?;

    let barrier = vk::ImageMemoryBarrier::builder()
        .image(destination_image)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();
    unsafe {
        aetna.device.cmd_pipeline_barrier(
            copybuffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };

    let source_image = aetna.swapchain.images[aetna.swapchain.current_image];
    let barrier = vk::ImageMemoryBarrier::builder()
        .image(source_image)
        .src_access_mask(vk::AccessFlags::MEMORY_READ)
        .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
        .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();
    unsafe {
        aetna.device.cmd_pipeline_barrier(
            copybuffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };

    let zero_offset = vk::Offset3D::default();
    let copy_area = vk::ImageCopy::builder()
        .src_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .src_offset(zero_offset)
        .dst_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .dst_offset(zero_offset)
        .extent(vk::Extent3D {
            width: aetna.swapchain.extent.width,
            height: aetna.swapchain.extent.height,
            depth: 1,
        })
        .build();
    unsafe {
        aetna.device.cmd_copy_image(
            copybuffer,
            source_image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            destination_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_area],
        )
    };

    // TODO(#7): Think about unnecessary barriers.
    let barrier = vk::ImageMemoryBarrier::builder()
        .image(destination_image)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::MEMORY_READ)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::GENERAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();
    unsafe {
        aetna.device.cmd_pipeline_barrier(
            copybuffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };

    let barrier = vk::ImageMemoryBarrier::builder()
        .image(source_image)
        .src_access_mask(vk::AccessFlags::TRANSFER_READ)
        .dst_access_mask(vk::AccessFlags::MEMORY_READ)
        .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();
    unsafe {
        aetna.device.cmd_pipeline_barrier(
            copybuffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };

    unsafe { aetna.device.end_command_buffer(copybuffer) }?;
    let submit_infos = [vk::SubmitInfo::builder()
        .command_buffers(&[copybuffer])
        .build()];
    let fence = unsafe {
        aetna
            .device
            .create_fence(&vk::FenceCreateInfo::default(), None)
    }?;
    unsafe {
        aetna
            .device
            .queue_submit(aetna.queues.graphics_queue, &submit_infos, fence)
    }?;
    unsafe { aetna.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
    unsafe { aetna.device.destroy_fence(fence, None) };
    unsafe {
        aetna
            .device
            .free_command_buffers(aetna.pools.commandpool_graphics, &[copybuffer])
    };

    let source_ptr = aetna.allocator.map_memory(&dst_alloc)? as *mut u8;
    let subresource_layout = unsafe {
        aetna.device.get_image_subresource_layout(
            destination_image,
            vk::ImageSubresource {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                array_layer: 0,
            },
        )
    };
    let mut data = Vec::<u8>::with_capacity(subresource_layout.size as usize);
    unsafe {
        std::ptr::copy(
            source_ptr,
            data.as_mut_ptr(),
            subresource_layout.size as usize,
        );
        data.set_len(subresource_layout.size as usize);
    }
    aetna.allocator.unmap_memory(&dst_alloc)?;
    aetna
        .allocator
        .destroy_image(destination_image, &dst_alloc)?;
    let screen: image::ImageBuffer<image::Bgra<u8>, _> = image::ImageBuffer::from_raw(
        aetna.swapchain.extent.width,
        aetna.swapchain.extent.height,
        data,
    )
    .expect("ImageBuffer creation");

    let screen_image = image::DynamicImage::ImageBgra8(screen).to_rgba8();
    screen_image.save("screenshot.jpg")?;

    Ok(())
}
