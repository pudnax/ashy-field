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
    let mut cube = model::Model::<model::VertexData, model::InstanceData>::sphere(3);
    cube.insert_visibly(model::InstanceData::from_matrix_and_colour(
        na::Matrix4::new_scaling(0.5),
        [0.5, 0.0, 0.2],
    ));
    cube.update_vertexbuffer(&aetna.allocator)?;
    cube.update_indexbuffer(&aetna.allocator)?;
    cube.update_instancebuffer(&aetna.allocator)?;
    aetna.models = vec![cube];
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
